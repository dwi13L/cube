use crate::{
    compile::rewrite::{
        analysis::Member,
        column_expr,
        rewriter::{CubeEGraph, CubeRewrite},
        rules::wrapper::WrapperRules,
        transforming_rewrite, wrapper_pullup_replacer, wrapper_pushdown_replacer, ColumnExprColumn,
        LogicalPlanLanguage, WrapperPullupReplacerAliasToCube, WrapperPullupReplacerUngroupedScan,
        WrapperPushdownReplacerUngroupedScan,
    },
    copy_flag, var, var_iter,
};
use egg::Subst;

impl WrapperRules {
    pub fn column_rules(&self, rules: &mut Vec<CubeRewrite>) {
        rules.extend(vec![
            transforming_rewrite(
                "wrapper-push-down-column",
                wrapper_pushdown_replacer(
                    column_expr("?name"),
                    "?alias_to_cube",
                    "WrapperPushdownReplacerPushToCube:false",
                    "?ungrouped_scan",
                    "?in_projection",
                    "?cube_members",
                ),
                wrapper_pullup_replacer(
                    column_expr("?name"),
                    "?alias_to_cube",
                    "WrapperPullupReplacerPushToCube:false",
                    "?pullup_ungrouped_scan",
                    "?in_projection",
                    "?cube_members",
                ),
                self.transform_column("?ungrouped_scan", "?pullup_ungrouped_scan"),
            ),
            // TODO This is half measure implementation to propagate ungrouped simple measure towards aggregate node that easily allow replacement of aggregation functions
            // We need to support it for complex aka `number` measures
            transforming_rewrite(
                "wrapper-push-down-column-simple-measure-in-projection",
                wrapper_pushdown_replacer(
                    column_expr("?name"),
                    "?alias_to_cube",
                    "WrapperPushdownReplacerPushToCube:true",
                    "?ungrouped_scan",
                    "WrapperPullupReplacerInProjection:true",
                    "?cube_members",
                ),
                wrapper_pullup_replacer(
                    column_expr("?name"),
                    "?alias_to_cube",
                    "WrapperPullupReplacerPushToCube:true",
                    "?pullup_ungrouped_scan",
                    "WrapperPullupReplacerInProjection:true",
                    "?cube_members",
                ),
                self.pushdown_simple_measure(
                    "?name",
                    "?cube_members",
                    "?ungrouped_scan",
                    "?pullup_ungrouped_scan",
                ),
            ),
            // TODO time dimension support
            transforming_rewrite(
                "wrapper-push-down-dimension",
                wrapper_pushdown_replacer(
                    column_expr("?name"),
                    "?alias_to_cube",
                    "WrapperPushdownReplacerPushToCube:true",
                    "?ungrouped_scan",
                    "?in_projection",
                    "?cube_members",
                ),
                wrapper_pullup_replacer(
                    "?dimension",
                    "?alias_to_cube",
                    "WrapperPullupReplacerPushToCube:true",
                    "?pullup_ungrouped_scan",
                    "?in_projection",
                    "?cube_members",
                ),
                self.pushdown_dimension(
                    "?alias_to_cube",
                    "?name",
                    "?cube_members",
                    "?dimension",
                    "?ungrouped_scan",
                    "?pullup_ungrouped_scan",
                ),
            ),
        ]);
    }

    fn pushdown_dimension(
        &self,
        alias_to_cube_var: &'static str,
        column_name_var: &'static str,
        members_var: &'static str,
        dimension_var: &'static str,
        ungrouped_scan_var: &'static str,
        pullup_ungrouped_scan_var: &'static str,
    ) -> impl Fn(&mut CubeEGraph, &mut Subst) -> bool {
        let alias_to_cube_var = var!(alias_to_cube_var);
        let column_name_var = var!(column_name_var);
        let members_var = var!(members_var);
        let dimension_var = var!(dimension_var);
        let ungrouped_scan_var = var!(ungrouped_scan_var);
        let pullup_ungrouped_scan_var = var!(pullup_ungrouped_scan_var);
        move |egraph, subst| {
            if !copy_flag!(
                egraph,
                subst,
                ungrouped_scan_var,
                WrapperPushdownReplacerUngroupedScan,
                pullup_ungrouped_scan_var,
                WrapperPullupReplacerUngroupedScan
            ) {
                return false;
            }
            let columns: Vec<_> = var_iter!(egraph[subst[column_name_var]], ColumnExprColumn)
                .cloned()
                .collect();
            for column in columns.iter() {
                for alias_to_cube in var_iter!(
                    egraph[subst[alias_to_cube_var]],
                    WrapperPullupReplacerAliasToCube
                )
                .cloned()
                {
                    //FIXME We always add subquery column as dimension. I'm not 100% sure that this is the correct solution
                    if let Some(col_relation) = &column.relation {
                        if &alias_to_cube[0].0 != col_relation
                            && col_relation.starts_with("__subquery")
                        {
                            let column_expr_column =
                                egraph.add(LogicalPlanLanguage::ColumnExprColumn(
                                    ColumnExprColumn(column.clone()),
                                ));

                            let column_expr =
                                egraph.add(LogicalPlanLanguage::ColumnExpr([column_expr_column]));
                            subst.insert(dimension_var, column_expr);
                            return true;
                        }
                    }
                }
                if let Some((member, _)) = &egraph[subst[members_var]]
                    .data
                    .find_member_by_alias(&column.name)
                {
                    if matches!(
                        member.1,
                        Member::Dimension { .. }
                            | Member::TimeDimension { .. }
                            | Member::Segment { .. }
                            | Member::ChangeUser { .. }
                            | Member::VirtualField { .. }
                            | Member::LiteralMember { .. }
                    ) {
                        let column_expr_column = egraph.add(LogicalPlanLanguage::ColumnExprColumn(
                            ColumnExprColumn(column.clone()),
                        ));

                        let column_expr =
                            egraph.add(LogicalPlanLanguage::ColumnExpr([column_expr_column]));
                        subst.insert(dimension_var, column_expr);
                        return true;
                    }
                }
            }
            false
        }
    }

    fn transform_column(
        &self,
        ungrouped_scan_var: &'static str,
        pullup_ungrouped_scan_var: &'static str,
    ) -> impl Fn(&mut CubeEGraph, &mut Subst) -> bool {
        let ungrouped_scan_var = var!(ungrouped_scan_var);
        let pullup_ungrouped_scan_var = var!(pullup_ungrouped_scan_var);
        move |egraph, subst| {
            if !copy_flag!(
                egraph,
                subst,
                ungrouped_scan_var,
                WrapperPushdownReplacerUngroupedScan,
                pullup_ungrouped_scan_var,
                WrapperPullupReplacerUngroupedScan
            ) {
                return false;
            }

            true
        }
    }

    fn pushdown_simple_measure(
        &self,
        column_name_var: &'static str,
        members_var: &'static str,
        ungrouped_scan_var: &'static str,
        pullup_ungrouped_scan_var: &'static str,
    ) -> impl Fn(&mut CubeEGraph, &mut Subst) -> bool {
        let column_name_var = var!(column_name_var);
        let members_var = var!(members_var);
        let ungrouped_scan_var = var!(ungrouped_scan_var);
        let pullup_ungrouped_scan_var = var!(pullup_ungrouped_scan_var);
        let meta = self.meta_context.clone();
        move |egraph, subst| {
            if !copy_flag!(
                egraph,
                subst,
                ungrouped_scan_var,
                WrapperPushdownReplacerUngroupedScan,
                pullup_ungrouped_scan_var,
                WrapperPullupReplacerUngroupedScan
            ) {
                return false;
            }

            let columns: Vec<_> = var_iter!(egraph[subst[column_name_var]], ColumnExprColumn)
                .cloned()
                .collect();
            for column in columns {
                if let Some(((Some(member), _, _), _)) = egraph[subst[members_var]]
                    .data
                    .find_member_by_alias(&column.name)
                {
                    if let Some(measure) = meta.find_measure_with_name(member.to_string()) {
                        if measure.agg_type != Some("number".to_string()) {
                            return true;
                        }
                    }
                }
            }
            false
        }
    }
}
