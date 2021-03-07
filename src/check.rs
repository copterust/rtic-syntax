use std::collections::HashSet;

use proc_macro2::Span;
use syn::parse;

use crate::ast::App;

pub fn app(app: &App) -> parse::Result<()> {
    // Check that all referenced resources have been declared
    // Check that resources are NOT `Exclusive`-ly shared
    let mut owners = HashSet::new();
    for (_, name, access) in app.resource_accesses() {
        if app.resource(name).is_none() {
            return Err(parse::Error::new(
                name.span(),
                "this resource has NOT been declared",
            ));
        }

        if access.is_exclusive() {
            owners.insert(name);
        }
    }

    // Check that no resource has both types of access (`Exclusive` & `Shared`)
    // TODO we want to allow this in the future (but behind a `Settings` feature gate)
    // accesses from `init` are not consider `Exclusive` accesses because `init` doesn't use the
    // `lock` API
    let exclusive_accesses = app
        .resource_accesses()
        .filter_map(|(priority, name, access)| {
            if priority.is_some() && access.is_exclusive() {
                Some(name)
            } else {
                None
            }
        })
        .collect::<HashSet<_>>();
    for (_, name, access) in app.resource_accesses() {
        if access.is_shared() && exclusive_accesses.contains(name) {
            return Err(parse::Error::new(
                name.span(),
                "this implementation doesn't support shared (`&-`) - exclusive (`&mut-`) locks; use `x` instead of `&x`",
            ));
        }
    }

    // Check that init only has `Access::Exclusive` resources
    // Check that late resources have NOT been assigned to `init`
    if let Some(init) = &app.inits.first() {
        for (name, access) in &init.args.resources {
            if app.late_resources.contains_key(name) {
                return Err(parse::Error::new(
                    name.span(),
                    "late resources can NOT be assigned to `init`",
                ));
            }

            if access.is_shared() {
                return Err(parse::Error::new(
                    name.span(),
                    "`init` has direct exclusive access to resources; use `x` instead of `&x` ",
                ));
            }
        }
    }

    // Check that all late resources are covered by `init::LateResources`
    let late_resources_set = app.late_resources.keys().collect::<HashSet<_>>();
    if !late_resources_set.is_empty() {
        // If there exist late_resources, check that #[init] returns them
        if app.inits.first().is_none() {
            return Err(parse::Error::new(
                Span::call_site(),
                "late resources exist so a `#[init]` function must be defined",
            ));
        }
    }

    // check that external interrupts are not used as hardware tasks
    for task in app.hardware_tasks.values() {
        let binds = &task.args.binds;

        for (bind, _) in binds.iter() {
            if app.args.extern_interrupts.contains_key(bind) {
                return Err(parse::Error::new(
                    bind.span(),
                    "dispatcher interrupts can't be used as hardware tasks",
                ));
            }
        }
    }

    Ok(())
}
