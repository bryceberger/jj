use std::rc::Rc;

use jj_lib::dsl_util;
use jj_lib::dsl_util::ExpressionNode;
use jj_lib::repo::ReadonlyRepo;
use jj_lib::revset::parse_program;
use jj_lib::revset::ExpressionKind;
use jj_lib::revset::ResolvedRevsetExpression;
use jj_lib::revset::RevsetDiagnostics;
use jj_lib::revset::RevsetExpression;
use jj_lib::revset::RevsetParseContext;
use jj_lib::revset::SymbolResolver;

use crate::cli_util::CommandHelper;
use crate::cli_util::RevisionArg;
use crate::command_error::print_parse_diagnostics;
use crate::command_error::CommandError;
use crate::revset_util;
use crate::ui::Ui;

/// Evaluate why revset is (or is not) contained in an expression
#[derive(clap::Args, Clone, Debug)]
pub struct DebugRevsetContainedInArgs {
    /// Target revision
    target: RevisionArg,
    /// Expression to debug
    expression: RevisionArg,
}

pub fn cmd_debug_revset_contained_in(
    ui: &mut Ui,
    command: &CommandHelper,
    args: &DebugRevsetContainedInArgs,
) -> Result<(), CommandError> {
    let workspace_command = command.workspace_helper(ui)?;
    let workspace_ctx = workspace_command.revset_parse_context();
    let repo = workspace_command.repo().as_ref();
    let mut symbol_resolver = revset_util::default_symbol_resolver(
        repo,
        command.revset_extensions().symbol_resolvers(),
        workspace_command.id_prefix_context(),
    );

    let target = workspace_command.resolve_single_rev(ui, &args.target)?;
    let target = RevsetExpression::commit(target.id().clone());

    let node = parse_program(args.expression.as_ref())?;
    let node: ExpressionNode<'_, ExpressionKind<'_>> =
        dsl_util::expand_aliases(node, workspace_ctx.aliases_map())?;
    show_contained_in(
        &mut ContainedInArgs {
            ui,
            context: &workspace_ctx,
            symbol_resolver: &mut symbol_resolver,
            repo,
            align: args.expression.as_ref().len(),
            target: &target,
        },
        &node,
        &mut "".into(),
    )?;

    Ok(())
}

struct ContainedInArgs<'a> {
    ui: &'a Ui,
    context: &'a RevsetParseContext<'a>,
    symbol_resolver: &'a mut dyn SymbolResolver,
    repo: &'a ReadonlyRepo,
    align: usize,
    target: &'a Rc<ResolvedRevsetExpression>,
}

fn show_contained_in(
    args: &mut ContainedInArgs<'_>,
    expression: &ExpressionNode<'_, ExpressionKind<'_>>,
    indent: &mut String,
) -> Result<(), CommandError> {
    args.align = args.align.max(expression.span.as_str().len());

    let mut diagnostics = RevsetDiagnostics::new();
    let revset_expression =
        jj_lib::revset::lower_expression(&mut diagnostics, expression, args.context)?;
    print_parse_diagnostics(args.ui, "In revset expression", &diagnostics)?;

    let contained_in = !revset_expression
        .resolve_user_expression(args.repo, args.symbol_resolver)?
        .intersection(args.target)
        .evaluate(args.repo)?
        .is_empty();
    let bold = "\x1b[1m";
    let dim = "\x1b[2m";
    let reset = "\x1b[0m";
    println!(
        "{color}{contained_in:5}{dim} {indent}{reset}{color}{expression}{reset}",
        expression = expression.span.as_str(),
        color = if contained_in { bold } else { dim }
    );

    if !indent.is_empty() && indent.ends_with("├─") {
        indent.replace_range(indent.len() - "├─".len().., "│ ");
    } else if indent.ends_with("└─") {
        indent.replace_range(indent.len() - "└─".len().., "  ");
    }

    match &expression.kind {
        ExpressionKind::Unary(_op, node) => {
            indent.push_str("└─");
            show_contained_in(args, node, indent)?;
            indent.pop();
            indent.pop();
            Ok(())
        }
        ExpressionKind::Binary(_op, left, right) => {
            indent.push_str("├─");
            show_contained_in(args, left, indent)?;
            indent.pop();
            indent.pop();
            indent.push_str("└─");
            show_contained_in(args, right, indent)?;
            indent.pop();
            indent.pop();
            Ok(())
        }
        ExpressionKind::UnionAll(vec) => {
            for (idx, expr) in vec.iter().enumerate() {
                let last = idx == vec.len() - 1;
                if last {
                    indent.push_str("└─");
                } else {
                    indent.push_str("├─");
                }
                show_contained_in(args, expr, indent)?;
                indent.pop();
                indent.pop();
            }
            Ok(())
        }
        ExpressionKind::AliasExpanded(_id, expanded) => {
            indent.push_str("└─");
            show_contained_in(args, expanded, indent)?;
            indent.pop();
            indent.pop();
            Ok(())
        }
        ExpressionKind::FunctionCall(call) => {
            for (idx, a) in call.args.iter().enumerate() {
                let last = idx == call.args.len() - 1;
                if last {
                    indent.push_str("└─");
                } else {
                    indent.push_str("├─");
                }
                show_contained_in(args, a, indent)?;
                indent.pop();
                indent.pop();
            }
            Ok(())
        }

        // terminals
        ExpressionKind::Identifier(_)
        | ExpressionKind::String(_)
        | ExpressionKind::StringPattern { .. }
        | ExpressionKind::RemoteSymbol { .. }
        | ExpressionKind::AtWorkspace(_)
        | ExpressionKind::AtCurrentWorkspace
        | ExpressionKind::DagRangeAll
        | ExpressionKind::RangeAll
        | ExpressionKind::Modifier(_) => Ok(()),
    }
}
