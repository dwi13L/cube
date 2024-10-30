use crate::{
    compile::rewrite::{
        cube_scan_wrapper,
        rewriter::{CubeEGraph, CubeRewrite},
        rules::wrapper::WrapperRules,
        transforming_rewrite, wrapped_select, wrapped_select_having_expr_empty_tail,
        wrapped_select_joins_empty_tail, wrapper_pullup_replacer, WrappedSelectSelectType,
        WrappedSelectType, WrappedSelectUngroupedScan, WrapperPullupReplacerUngroupedScan,
    },
    copy_flag, var, var_iter, var_list_iter,
};
use egg::Subst;

impl WrapperRules {
    pub fn wrapper_pull_up_rules(&self, rules: &mut Vec<CubeRewrite>) {
        rules.extend(vec![
            transforming_rewrite(
                "wrapper-pull-up-to-cube-scan-non-wrapped-select",
                cube_scan_wrapper(
                    wrapped_select(
                        "?select_type",
                        wrapper_pullup_replacer(
                            "?projection_expr",
                            "?alias_to_cube",
                            "?push_to_cube",
                            "?ungrouped_scan",
                            "?in_projection",
                            "?cube_members",
                        ),
                        wrapper_pullup_replacer(
                            "?subqueries",
                            "?alias_to_cube",
                            "?push_to_cube",
                            "?ungrouped_scan",
                            "?in_projection",
                            "?cube_members",
                        ),
                        wrapper_pullup_replacer(
                            "?group_expr",
                            "?alias_to_cube",
                            "?push_to_cube",
                            "?ungrouped_scan",
                            "?in_projection",
                            "?cube_members",
                        ),
                        wrapper_pullup_replacer(
                            "?aggr_expr",
                            "?alias_to_cube",
                            "?push_to_cube",
                            "?ungrouped_scan",
                            "?in_projection",
                            "?cube_members",
                        ),
                        wrapper_pullup_replacer(
                            "?window_expr",
                            "?alias_to_cube",
                            "?push_to_cube",
                            "?ungrouped_scan",
                            "?in_projection",
                            "?cube_members",
                        ),
                        wrapper_pullup_replacer(
                            "?cube_scan_input",
                            "?alias_to_cube",
                            "?push_to_cube",
                            "?ungrouped_scan",
                            "?in_projection",
                            "?cube_members",
                        ),
                        wrapped_select_joins_empty_tail(),
                        wrapper_pullup_replacer(
                            "?filter_expr",
                            "?alias_to_cube",
                            "?push_to_cube",
                            "?ungrouped_scan",
                            "?in_projection",
                            "?cube_members",
                        ),
                        wrapped_select_having_expr_empty_tail(),
                        "WrappedSelectLimit:None",
                        "WrappedSelectOffset:None",
                        wrapper_pullup_replacer(
                            "?order_expr",
                            "?alias_to_cube",
                            "?push_to_cube",
                            "?ungrouped_scan",
                            "?in_projection",
                            "?cube_members",
                        ),
                        "?select_alias",
                        "?select_distinct",
                        "?select_push_to_cube",
                        "?select_ungrouped_scan",
                    ),
                    "CubeScanWrapperFinalized:false",
                ),
                cube_scan_wrapper(
                    wrapper_pullup_replacer(
                        wrapped_select(
                            "?select_type",
                            "?projection_expr",
                            "?subqueries",
                            "?group_expr",
                            "?aggr_expr",
                            "?window_expr",
                            "?cube_scan_input",
                            wrapped_select_joins_empty_tail(),
                            "?filter_expr",
                            wrapped_select_having_expr_empty_tail(),
                            "WrappedSelectLimit:None",
                            "WrappedSelectOffset:None",
                            "?order_expr",
                            "?select_alias",
                            "?select_distinct",
                            "?select_push_to_cube",
                            "?select_ungrouped_scan",
                        ),
                        "?alias_to_cube",
                        // This is fixed to false for any LHS because we should only allow to push to Cube when from is ungrouped CubeScan
                        // And after pulling replacer over this node it will be WrappedSelect(from=CubeScan), so it should not allow to push for whatever LP is on top of it
                        "WrapperPullupReplacerPushToCube:false",
                        "?ungrouped_scan_out",
                        "?in_projection",
                        "?cube_members",
                    ),
                    "CubeScanWrapperFinalized:false",
                ),
                self.transform_pull_up_wrapper_select(
                    "?cube_scan_input",
                    "?select_ungrouped_scan",
                    "?ungrouped_scan_out",
                ),
            ),
            // This rule would introduce new representations:
            // replacers from node on top of this node would be able to run with enabled push to Cube.
            // However, those representation are not valid on their own, they are necessary
            // for flattening to work.
            // So, they will be disallowed to pullup by nontrivial pullup rules.
            // Also note that trivial pullup rules are allowed to pullup without additional checks,
            // because it checks that `from` is not WrappedSelect, so push=true in input can be valid
            // There's no equivalent for non-trivial pullup, because it has WrappedSelect in `from`,
            // so there's no way to use push to Cube even after flattening
            transforming_rewrite(
                "wrapper-pull-up-to-cube-scan-non-wrapped-select-with-push",
                cube_scan_wrapper(
                    wrapped_select(
                        "?select_type",
                        wrapper_pullup_replacer(
                            "?projection_expr",
                            "?alias_to_cube",
                            "?push_to_cube",
                            "?ungrouped_scan",
                            "?in_projection",
                            "?cube_members",
                        ),
                        wrapper_pullup_replacer(
                            "?subqueries",
                            "?alias_to_cube",
                            "?push_to_cube",
                            "?ungrouped_scan",
                            "?in_projection",
                            "?cube_members",
                        ),
                        wrapper_pullup_replacer(
                            "?group_expr",
                            "?alias_to_cube",
                            "?push_to_cube",
                            "?ungrouped_scan",
                            "?in_projection",
                            "?cube_members",
                        ),
                        wrapper_pullup_replacer(
                            "?aggr_expr",
                            "?alias_to_cube",
                            "?push_to_cube",
                            "?ungrouped_scan",
                            "?in_projection",
                            "?cube_members",
                        ),
                        wrapper_pullup_replacer(
                            "?window_expr",
                            "?alias_to_cube",
                            "?push_to_cube",
                            "?ungrouped_scan",
                            "?in_projection",
                            "?cube_members",
                        ),
                        wrapper_pullup_replacer(
                            "?cube_scan_input",
                            "?alias_to_cube",
                            "?push_to_cube",
                            "?ungrouped_scan",
                            "?in_projection",
                            "?cube_members",
                        ),
                        wrapped_select_joins_empty_tail(),
                        wrapper_pullup_replacer(
                            "?filter_expr",
                            "?alias_to_cube",
                            "?push_to_cube",
                            "?ungrouped_scan",
                            "?in_projection",
                            "?cube_members",
                        ),
                        wrapped_select_having_expr_empty_tail(),
                        "WrappedSelectLimit:None",
                        "WrappedSelectOffset:None",
                        wrapper_pullup_replacer(
                            "?order_expr",
                            "?alias_to_cube",
                            "?push_to_cube",
                            "?ungrouped_scan",
                            "?in_projection",
                            "?cube_members",
                        ),
                        "?select_alias",
                        "?select_distinct",
                        "WrappedSelectPushToCube:true",
                        "WrappedSelectUngroupedScan:true",
                    ),
                    "CubeScanWrapperFinalized:false",
                ),
                cube_scan_wrapper(
                    wrapper_pullup_replacer(
                        wrapped_select(
                            "?select_type",
                            "?projection_expr",
                            "?subqueries",
                            "?group_expr",
                            "?aggr_expr",
                            "?window_expr",
                            "?cube_scan_input",
                            wrapped_select_joins_empty_tail(),
                            "?filter_expr",
                            wrapped_select_having_expr_empty_tail(),
                            "WrappedSelectLimit:None",
                            "WrappedSelectOffset:None",
                            "?order_expr",
                            "?select_alias",
                            "?select_distinct",
                            "WrappedSelectPushToCube:true",
                            "WrappedSelectUngroupedScan:true",
                        ),
                        "?alias_to_cube",
                        // This would allow next LP node on top of this to use push even when from=WrappedSelect(from=CubeScan)
                        // Wrapper like that would be incorrect, so they will be disallowed by pullup rules for WrappedSelect(from=WrappedSelect) case
                        "WrapperPullupReplacerPushToCube:true",
                        "WrapperPullupReplacerUngroupedScan:true",
                        "?in_projection",
                        "?cube_members",
                    ),
                    "CubeScanWrapperFinalized:false",
                ),
                self.transform_pull_up_wrapper_select_for_push("?cube_scan_input"),
            ),
            transforming_rewrite(
                "wrapper-pull-up-to-cube-scan-non-trivial-wrapped-select",
                cube_scan_wrapper(
                    wrapped_select(
                        "?select_type",
                        wrapper_pullup_replacer(
                            "?projection_expr",
                            "?alias_to_cube",
                            "?push_to_cube",
                            "?ungrouped_scan",
                            "?in_projection",
                            "?cube_members",
                        ),
                        wrapper_pullup_replacer(
                            "?subqueries",
                            "?alias_to_cube",
                            "?push_to_cube",
                            "?ungrouped_scan",
                            "?in_projection",
                            "?cube_members",
                        ),
                        wrapper_pullup_replacer(
                            "?group_expr",
                            "?alias_to_cube",
                            "?push_to_cube",
                            "?ungrouped_scan",
                            "?in_projection",
                            "?cube_members",
                        ),
                        wrapper_pullup_replacer(
                            "?aggr_expr",
                            "?alias_to_cube",
                            "?push_to_cube",
                            "?ungrouped_scan",
                            "?in_projection",
                            "?cube_members",
                        ),
                        wrapper_pullup_replacer(
                            "?window_expr",
                            "?alias_to_cube",
                            "?push_to_cube",
                            "?ungrouped_scan",
                            "?in_projection",
                            "?cube_members",
                        ),
                        wrapper_pullup_replacer(
                            wrapped_select(
                                "?inner_select_type",
                                "?inner_projection_expr",
                                "?inner_subqueries",
                                "?inner_group_expr",
                                "?inner_aggr_expr",
                                "?inner_window_expr",
                                "?inner_cube_scan_input",
                                "?inner_joins",
                                "?inner_filter_expr",
                                "?inner_having_expr",
                                "?inner_limit",
                                "?inner_offset",
                                "?inner_order_expr",
                                "?inner_alias",
                                "?inner_distinct",
                                "?inner_push_to_cube",
                                "?inner_ungrouped_scan",
                            ),
                            "?alias_to_cube",
                            "?push_to_cube",
                            "?ungrouped_scan",
                            "?in_projection",
                            "?cube_members",
                        ),
                        wrapped_select_joins_empty_tail(),
                        wrapper_pullup_replacer(
                            "?filter_expr",
                            "?alias_to_cube",
                            "?push_to_cube",
                            "?ungrouped_scan",
                            "?in_projection",
                            "?cube_members",
                        ),
                        wrapped_select_having_expr_empty_tail(),
                        "WrappedSelectLimit:None",
                        "WrappedSelectOffset:None",
                        wrapper_pullup_replacer(
                            "?order_expr",
                            "?alias_to_cube",
                            "?push_to_cube",
                            "?ungrouped_scan",
                            "?in_projection",
                            "?cube_members",
                        ),
                        "?select_alias",
                        "?select_distinct",
                        // This node has a WrappedSelect in from, so it's not allowed to use push to Cube
                        "WrappedSelectPushToCube:false",
                        "?select_ungrouped_scan",
                    ),
                    "CubeScanWrapperFinalized:false",
                ),
                cube_scan_wrapper(
                    wrapper_pullup_replacer(
                        wrapped_select(
                            "?select_type",
                            "?projection_expr",
                            "?subqueries",
                            "?group_expr",
                            "?aggr_expr",
                            "?window_expr",
                            wrapped_select(
                                "?inner_select_type",
                                "?inner_projection_expr",
                                "?inner_subqueries",
                                "?inner_group_expr",
                                "?inner_aggr_expr",
                                "?inner_window_expr",
                                "?inner_cube_scan_input",
                                "?inner_joins",
                                "?inner_filter_expr",
                                "?inner_having_expr",
                                "?inner_limit",
                                "?inner_offset",
                                "?inner_order_expr",
                                "?inner_alias",
                                "?inner_distinct",
                                "?inner_push_to_cube",
                                "?inner_ungrouped_scan",
                            ),
                            wrapped_select_joins_empty_tail(),
                            "?filter_expr",
                            wrapped_select_having_expr_empty_tail(),
                            "WrappedSelectLimit:None",
                            "WrappedSelectOffset:None",
                            "?order_expr",
                            "?select_alias",
                            "?select_distinct",
                            "WrappedSelectPushToCube:false",
                            "?select_ungrouped_scan",
                        ),
                        "?alias_to_cube",
                        // This is fixed to false for any LHS because we should only allow to push to Cube when from is ungrouped CubeSCan
                        // And after pulling replacer over this node it will be WrappedSelect(from=WrappedSelect), so it should not allow to push for whatever LP is on top of it
                        "WrapperPullupReplacerPushToCube:false",
                        "?ungrouped_scan_out",
                        "?inner_projection_expr",
                        "?cube_members",
                    ),
                    "CubeScanWrapperFinalized:false",
                ),
                self.transform_pull_up_non_trivial_wrapper_select(
                    "?select_type",
                    "?projection_expr",
                    "?group_expr",
                    "?aggr_expr",
                    "?inner_select_type",
                    "?inner_projection_expr",
                    "?inner_group_expr",
                    "?inner_aggr_expr",
                    "?select_ungrouped_scan",
                    "?ungrouped_scan_out",
                ),
            ),
        ]);
    }

    fn transform_pull_up_wrapper_select(
        &self,
        cube_scan_input_var: &'static str,
        select_ungrouped_scan_var: &'static str,
        ungrouped_scan_out_var: &'static str,
    ) -> impl Fn(&mut CubeEGraph, &mut Subst) -> bool {
        let cube_scan_input_var = var!(cube_scan_input_var);
        let select_ungrouped_scan_var = var!(select_ungrouped_scan_var);
        let ungrouped_scan_out_var = var!(ungrouped_scan_out_var);
        move |egraph, subst| {
            for _ in var_list_iter!(egraph[subst[cube_scan_input_var]], WrappedSelect).cloned() {
                return false;
            }

            if !copy_flag!(
                egraph,
                subst,
                select_ungrouped_scan_var,
                WrappedSelectUngroupedScan,
                ungrouped_scan_out_var,
                WrapperPullupReplacerUngroupedScan
            ) {
                return false;
            }

            true
        }
    }

    fn transform_pull_up_wrapper_select_for_push(
        &self,
        cube_scan_input_var: &'static str,
    ) -> impl Fn(&mut CubeEGraph, &mut Subst) -> bool {
        let cube_scan_input_var = var!(cube_scan_input_var);
        move |egraph, subst| {
            for _ in var_list_iter!(egraph[subst[cube_scan_input_var]], WrappedSelect).cloned() {
                return false;
            }

            true
        }
    }

    fn transform_pull_up_non_trivial_wrapper_select(
        &self,
        select_type_var: &'static str,
        projection_expr_var: &'static str,
        _group_expr_var: &'static str,
        _aggr_expr_var: &'static str,
        inner_select_type_var: &'static str,
        inner_projection_expr_var: &'static str,
        _inner_group_expr_var: &'static str,
        _inner_aggr_expr_var: &'static str,
        select_ungrouped_scan_var: &'static str,
        ungrouped_scan_out_var: &'static str,
    ) -> impl Fn(&mut CubeEGraph, &mut Subst) -> bool {
        let select_type_var = var!(select_type_var);
        let projection_expr_var = var!(projection_expr_var);
        let inner_select_type_var = var!(inner_select_type_var);
        let inner_projection_expr_var = var!(inner_projection_expr_var);
        let select_ungrouped_scan_var = var!(select_ungrouped_scan_var);
        let ungrouped_scan_out_var = var!(ungrouped_scan_out_var);
        move |egraph, subst| {
            if !copy_flag!(
                egraph,
                subst,
                select_ungrouped_scan_var,
                WrappedSelectUngroupedScan,
                ungrouped_scan_out_var,
                WrapperPullupReplacerUngroupedScan
            ) {
                return false;
            }

            for select_type in
                var_iter!(egraph[subst[select_type_var]], WrappedSelectSelectType).cloned()
            {
                for inner_select_type in var_iter!(
                    egraph[subst[inner_select_type_var]],
                    WrappedSelectSelectType
                )
                .cloned()
                {
                    if select_type != inner_select_type {
                        return true;
                    }

                    return match select_type {
                        WrappedSelectType::Projection => {
                            // TODO changes of alias can be non-trivial
                            subst[projection_expr_var] != subst[inner_projection_expr_var]
                        }
                        WrappedSelectType::Aggregate => {
                            // TODO write rules for non trivial wrapped aggregate
                            true
                        }
                    };
                }
            }
            false
        }
    }
}
