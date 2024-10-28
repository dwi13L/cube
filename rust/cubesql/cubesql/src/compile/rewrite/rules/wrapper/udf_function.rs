use crate::{
    compile::rewrite::{
        rewrite,
        rewriter::{CubeEGraph, CubeRewrite},
        rules::wrapper::WrapperRules,
        transforming_rewrite, udf_expr_var_arg, udf_fun_expr_args, udf_fun_expr_args_empty_tail,
        wrapper_pullup_replacer, wrapper_pushdown_replacer, ScalarUDFExprFun,
        WrapperPullupReplacerAliasToCube, WrapperPullupReplacerPushToCube,
        WrapperPullupReplacerUngroupedScan, WrapperPushdownReplacerPushToCube,
        WrapperPushdownReplacerUngroupedScan,
    },
    copy_flag, var, var_iter,
};
use egg::Subst;

impl WrapperRules {
    pub fn udf_function_rules(&self, rules: &mut Vec<CubeRewrite>) {
        rules.extend(vec![
            rewrite(
                "wrapper-push-down-udf",
                wrapper_pushdown_replacer(
                    udf_expr_var_arg("?fun", "?args"),
                    "?alias_to_cube",
                    "?push_to_cube",
                    "?ungrouped_scan",
                    "?in_projection",
                    "?cube_members",
                ),
                udf_expr_var_arg(
                    "?fun",
                    wrapper_pushdown_replacer(
                        "?args",
                        "?alias_to_cube",
                        "?push_to_cube",
                        "?ungrouped_scan",
                        "?in_projection",
                        "?cube_members",
                    ),
                ),
            ),
            transforming_rewrite(
                "wrapper-pull-up-udf",
                udf_expr_var_arg(
                    "?fun",
                    wrapper_pullup_replacer(
                        "?args",
                        "?alias_to_cube",
                        "?push_to_cube",
                        "?ungrouped_scan",
                        "?in_projection",
                        "?cube_members",
                    ),
                ),
                wrapper_pullup_replacer(
                    udf_expr_var_arg("?fun", "?args"),
                    "?alias_to_cube",
                    "?push_to_cube",
                    "?ungrouped_scan",
                    "?in_projection",
                    "?cube_members",
                ),
                self.transform_udf_expr("?fun", "?alias_to_cube"),
            ),
            rewrite(
                "wrapper-push-down-udf-args",
                wrapper_pushdown_replacer(
                    udf_fun_expr_args("?left", "?right"),
                    "?alias_to_cube",
                    "?push_to_cube",
                    "?ungrouped_scan",
                    "?in_projection",
                    "?cube_members",
                ),
                udf_fun_expr_args(
                    wrapper_pushdown_replacer(
                        "?left",
                        "?alias_to_cube",
                        "?push_to_cube",
                        "?ungrouped_scan",
                        "?in_projection",
                        "?cube_members",
                    ),
                    wrapper_pushdown_replacer(
                        "?right",
                        "?alias_to_cube",
                        "?push_to_cube",
                        "?ungrouped_scan",
                        "?in_projection",
                        "?cube_members",
                    ),
                ),
            ),
            rewrite(
                "wrapper-pull-up-udf-args",
                udf_fun_expr_args(
                    wrapper_pullup_replacer(
                        "?left",
                        "?alias_to_cube",
                        "?push_to_cube",
                        "?ungrouped_scan",
                        "?in_projection",
                        "?cube_members",
                    ),
                    wrapper_pullup_replacer(
                        "?right",
                        "?alias_to_cube",
                        "?push_to_cube",
                        "?ungrouped_scan",
                        "?in_projection",
                        "?cube_members",
                    ),
                ),
                wrapper_pullup_replacer(
                    udf_fun_expr_args("?left", "?right"),
                    "?alias_to_cube",
                    "?push_to_cube",
                    "?ungrouped_scan",
                    "?in_projection",
                    "?cube_members",
                ),
            ),
            transforming_rewrite(
                "wrapper-push-down-udf-empty-tail",
                wrapper_pushdown_replacer(
                    udf_fun_expr_args_empty_tail(),
                    "?alias_to_cube",
                    "?push_to_cube",
                    "?ungrouped_scan",
                    "?in_projection",
                    "?cube_members",
                ),
                wrapper_pullup_replacer(
                    udf_fun_expr_args_empty_tail(),
                    "?alias_to_cube",
                    "?pullup_push_to_cube",
                    "?pullup_ungrouped_scan",
                    "?in_projection",
                    "?cube_members",
                ),
                self.transform_udf_expr_tail(
                    "?push_to_cube",
                    "?pullup_push_to_cube",
                    "?ungrouped_scan",
                    "?pullup_ungrouped_scan",
                ),
            ),
        ]);
    }

    fn transform_udf_expr(
        &self,
        fun_var: &'static str,
        alias_to_cube_var: &'static str,
    ) -> impl Fn(&mut CubeEGraph, &mut Subst) -> bool {
        let fun_var = var!(fun_var);
        let alias_to_cube_var = var!(alias_to_cube_var);
        let meta = self.meta_context.clone();
        move |egraph, subst| {
            for alias_to_cube in var_iter!(
                egraph[subst[alias_to_cube_var]],
                WrapperPullupReplacerAliasToCube
            )
            .cloned()
            {
                if let Some(sql_generator) = meta.sql_generator_by_alias_to_cube(&alias_to_cube) {
                    for fun in var_iter!(egraph[subst[fun_var]], ScalarUDFExprFun).cloned() {
                        if sql_generator
                            .get_sql_templates()
                            .templates
                            .contains_key(&format!("functions/{}", fun.to_uppercase()))
                        {
                            return true;
                        }
                    }
                }
            }
            false
        }
    }

    fn transform_udf_expr_tail(
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
        move |egraph, subst| {
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
