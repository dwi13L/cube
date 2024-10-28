use crate::{
    compile::rewrite::{
        insubquery_expr, rewrite,
        rewriter::{CubeEGraph, CubeRewrite},
        rules::wrapper::WrapperRules,
        transforming_rewrite, wrapper_pullup_replacer, wrapper_pushdown_replacer,
        WrapperPullupReplacerPushToCube, WrapperPullupReplacerUngroupedScan,
        WrapperPushdownReplacerPushToCube, WrapperPushdownReplacerUngroupedScan,
    },
    copy_flag, var,
};
use egg::Subst;

impl WrapperRules {
    pub fn in_subquery_expr_rules(&self, rules: &mut Vec<CubeRewrite>) {
        rules.extend(vec![
            transforming_rewrite(
                "wrapper-in-subquery-push-down",
                wrapper_pushdown_replacer(
                    insubquery_expr("?expr", "?subquery", "?negated"),
                    "?alias_to_cube",
                    "?push_to_cube",
                    "?ungrouped_scan",
                    "?in_projection",
                    "?cube_members",
                ),
                insubquery_expr(
                    wrapper_pushdown_replacer(
                        "?expr",
                        "?alias_to_cube",
                        "?push_to_cube",
                        "?ungrouped_scan",
                        "?in_projection",
                        "?cube_members",
                    ),
                    wrapper_pullup_replacer(
                        "?subquery",
                        "?alias_to_cube",
                        "?pullup_push_to_cube",
                        "?pullup_ungrouped_scan",
                        "?in_projection",
                        "?cube_members",
                    ),
                    "?negated",
                ),
                self.transform_in_subquery_pushdown(
                    "?push_to_cube",
                    "?pullup_push_to_cube",
                    "?ungrouped_scan",
                    "?pullup_ungrouped_scan",
                ),
            ),
            rewrite(
                "wrapper-in-subquery-pull-up",
                insubquery_expr(
                    wrapper_pullup_replacer(
                        "?expr",
                        "?alias_to_cube",
                        "?push_to_cube",
                        "?ungrouped_scan",
                        "?in_projection",
                        "?cube_members",
                    ),
                    wrapper_pullup_replacer(
                        "?subquery",
                        "?alias_to_cube",
                        "?push_to_cube",
                        "?ungrouped_scan",
                        "?in_projection",
                        "?cube_members",
                    ),
                    "?negated",
                ),
                wrapper_pullup_replacer(
                    insubquery_expr("?expr", "?subquery", "?negated"),
                    "?alias_to_cube",
                    "?push_to_cube",
                    "?ungrouped_scan",
                    "?in_projection",
                    "?cube_members",
                ),
            ),
        ]);
    }

    fn transform_in_subquery_pushdown(
        &self,
        push_to_cube_var: &'static str,
        pullup_push_to_cube_var: &'static str,
        ungrouped_scan_var: &'static str,
        pullup_ungrouped_scan_var: &'static str,
    ) -> impl Fn(&mut CubeEGraph, &mut Subst) -> bool {
        let push_to_cube_var = var!(push_to_cube_var);
        let pullup_push_to_cube_var = var!(pullup_push_to_cube_var);
        let ungrouped_scan_var = var!(ungrouped_scan_var);
        let pullup_ungrouped_scan_var = var!(pullup_ungrouped_scan_var);
        move |egraph: &mut CubeEGraph, subst| {
            if !copy_flag!(
                egraph,
                subst,
                push_to_cube_var,
                WrapperPushdownReplacerPushToCube,
                pullup_push_to_cube_var,
                WrapperPullupReplacerPushToCube
            ) {
                return false;
            }
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
}
