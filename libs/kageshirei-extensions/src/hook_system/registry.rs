//! The hook registry module provides a way to register and trigger hooks with metadata.

use std::{
    any::Any,
    collections::HashMap,
    fmt::{Debug, Formatter},
    future::Future,
    pin::Pin,
    sync::Arc,
};

use itertools::Itertools as _;
use tokio::sync::RwLock;

/// The type of hook function, this is a boxed function that takes a boxed `Any` and returns a
/// future (aka async function with arbitrary context)
type HookFn = Box<
    dyn Fn(Arc<Box<dyn Any + Send + Sync>>) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>> + Send + Sync,
>;

/// The metadata of a hook
#[derive(Debug, Clone)]
pub struct HookMetadata {
    /// The priority of the hook, lower runs first (0 is the highest priority)
    priority:    u8,
    /// The hook description
    description: String,
}

/// The structured representation of a hook
struct Hook {
    /// The metadata of the hook
    metadata: HookMetadata,
    /// The hook function
    hook:     HookFn,
}

impl Hook {
    /// Create a new hook
    pub fn new(metadata: HookMetadata, hook: HookFn) -> Self {
        Self {
            metadata,
            hook,
        }
    }

    /// Run the hook
    pub async fn run(&self, context: Arc<Box<dyn Any + Send + Sync>>) -> Result<(), String> {
        (self.hook)(context).await
    }

    /// Get the priority of the hook
    pub const fn priority(&self) -> u8 { self.metadata.priority }

    /// Get the description of the hook
    pub fn description(&self) -> String { self.metadata.description.clone() }
}

impl Debug for Hook {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Hook")
            .field("metadata", &self.metadata)
            .field("hook_type", &std::any::type_name::<HookFn>())
            .finish()
    }
}

/// A registry of hooks
#[derive(Debug)]
pub struct HookRegistry {
    /// A map of hooks to their names (e.g. "on_start")
    hooks: RwLock<HashMap<String, Vec<Hook>>>,
}

impl Default for HookRegistry {
    fn default() -> Self { Self::new() }
}

impl HookRegistry {
    /// Create a new hook registry
    pub fn new() -> Self {
        Self {
            hooks: RwLock::new(HashMap::new()),
        }
    }

    /// Register a hook
    pub async fn register<C, F, Fut>(&self, hook_id: &str, metadata: HookMetadata, hook: F)
    where
        C: 'static + Any + Send,
        F: Fn(&C) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), String>> + Send + 'static,
    {
        #![expect(
            clippy::significant_drop_tightening,
            reason = "The hooks cannot be dropped as they are used below and the vector must stay writable until the \
                      end of the function. ref to line: `let mut hooks = ...`"
        )]

        let hook = Arc::new(hook);

        // Lock the hooks map
        let mut hooks = self.hooks.write().await;

        // If the entry doesn't exist, create it
        let entry = hooks.entry(hook_id.to_owned()).or_insert_with(Vec::new);

        // Push the hook into the registry
        entry.push(Hook {
            metadata,
            hook: Box::new(move |context| {
                let hook = hook.clone(); // Clone the Arc to share ownership
                Box::pin(async move {
                    // Try to downcast the data to the expected type
                    let typed_context = context.downcast_ref::<C>().ok_or_else(|| {
                        format!(
                            "Hook expected context of type {}",
                            std::any::type_name::<C>()
                        )
                    })?;
                    hook(typed_context).await
                })
            }),
        });
    }

    /// Run all hooks for a given name
    pub async fn trigger<C>(&self, hood_id: &str, context: C) -> Result<(), Vec<String>>
    where
        C: 'static + Any + Send + Sync,
    {
        #![expect(
            clippy::significant_drop_tightening,
            reason = "The hooks are not dropped as they are used into the if-let construct below"
        )]
        let rw_locked_hooks = self.hooks.read().await;
        if let Some(hooks) = rw_locked_hooks.get(hood_id) {
            // Type-erase the context
            let context = Arc::new(Box::new(context) as Box<dyn Any + Send + Sync>);

            let mut errors = Vec::<String>::new();

            // Run all hooks
            for hook in hooks.iter().sorted_by_key(|hook| hook.priority()) {
                if let Err(e) = hook.run(context.clone()).await {
                    errors.push(e);
                }
            }

            return if errors.is_empty() {
                Ok(())
            }
            else {
                Err(errors)
            };
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use kageshirei_extensions_macros::registerable_hook;
    use tokio::sync::Mutex;

    use super::*;

    #[tokio::test]
    async fn test_hook_creation_and_execution() {
        // Shared state for validation
        let state = Arc::new(Mutex::new(0));

        // Metadata for the hook
        let metadata = HookMetadata {
            priority:    0,
            description: "Increment state by 1".to_string(),
        };

        // Hook function
        let hook_fn: HookFn = Box::new({
            let state = Arc::clone(&state);
            move |_| {
                Box::pin({
                    let value = state.clone();
                    async move {
                        let mut guard = value.lock().await;
                        *guard += 1;
                        Ok(())
                    }
                })
            }
        });

        // Create the hook
        let hook = Hook::new(metadata.clone(), hook_fn);

        // Run the hook
        let context = Arc::new(Box::new(()) as Box<dyn Any + Send + Sync>);
        assert!(hook.run(context).await.is_ok());

        // Verify state increment
        assert_eq!(*state.lock().await, 1);

        // Verify metadata
        assert_eq!(hook.priority(), metadata.priority);
        assert_eq!(hook.description(), metadata.description);

        // Debug output
        println!("{:?}", hook);
    }

    #[tokio::test]
    async fn test_hook_registry_registration_and_triggering() {
        // Create the registry
        let registry = HookRegistry::new();

        // Shared state for validation
        let state = Arc::new(Mutex::new(0));

        // Register hooks with different priorities
        registry
            .register(
                "on_event",
                HookMetadata {
                    priority:    1,
                    description: "Add 2 to state".to_string(),
                },
                {
                    let state = Arc::clone(&state);
                    move |_: &i32| {
                        Box::pin({
                            let value = state.clone();
                            async move {
                                let mut guard = value.lock().await;
                                *guard += 2;
                                Ok(())
                            }
                        })
                    }
                },
            )
            .await;

        registry
            .register(
                "on_event",
                HookMetadata {
                    priority:    0,
                    description: "Add 1 to state".to_string(),
                },
                {
                    let state = Arc::clone(&state);
                    move |_: &i32| {
                        Box::pin({
                            let value = state.clone();
                            async move {
                                let mut guard = value.lock().await;
                                *guard += 1;
                                Ok(())
                            }
                        })
                    }
                },
            )
            .await;

        // Trigger the hooks
        assert!(registry.trigger("on_event", 42).await.is_ok());

        // Verify execution order based on priority
        assert_eq!(*state.lock().await, 3);
    }

    #[tokio::test]
    async fn test_hook_registry_error_handling() {
        // Create the registry
        let registry = HookRegistry::new();

        // Register a failing hook
        registry
            .register(
                "on_fail",
                HookMetadata {
                    priority:    0,
                    description: "Failing hook".to_string(),
                },
                move |_: &i32| Box::pin(async { Err("Hook failed".to_string()) }),
            )
            .await;

        // Trigger the hooks
        let result = registry.trigger("on_fail", 42).await;

        // Verify error handling
        assert!(result.is_err());
        if let Err(errors) = result {
            assert_eq!(errors.len(), 1);
            assert_eq!(errors[0], "Hook failed");
        }
    }

    #[tokio::test]
    async fn test_hook_registry_with_metadata() {
        // Create the registry
        let registry = HookRegistry::new();

        // Metadata for validation
        let metadata = HookMetadata {
            priority:    5,
            description: "Metadata test hook".to_string(),
        };

        // Shared state for validation
        let state = Arc::new(Mutex::new(0));

        // Register a hook with metadata
        registry
            .register("metadata_event", metadata.clone(), {
                let state = Arc::clone(&state);
                move |_: &i32| {
                    Box::pin({
                        let value = state.clone();
                        async move {
                            let mut guard = value.lock().await;
                            *guard += 5;
                            Ok(())
                        }
                    })
                }
            })
            .await;

        // Trigger the hook
        assert!(registry.trigger("metadata_event", 42).await.is_ok());

        // Verify state update
        assert_eq!(*state.lock().await, 5);

        // Verify hooks in registry
        let hooks = registry.hooks.read().await;
        assert!(hooks.contains_key("metadata_event"));
        if let Some(hooks) = hooks.get("metadata_event") {
            assert_eq!(hooks.len(), 1);
            assert_eq!(hooks[0].metadata.priority, metadata.priority);
            assert_eq!(hooks[0].metadata.description, metadata.description);
        }
    }

    struct TestContext {
        value: Mutex<i32>,
    }
    async fn increase_context_value(context: Arc<Box<TestContext>>) -> Result<(), String> {
        let mut guard = context.value.lock().await;
        *guard += 1;

        Ok(())
    }
    #[tokio::test]
    async fn test_hook_registry_registration_with_standard_function() {
        let registry = HookRegistry::new();

        let context = Arc::new(Box::new(TestContext {
            value: Mutex::new(0),
        }));

        registry
            .register(
                "on_event",
                HookMetadata {
                    priority:    0,
                    description: "Increment context value by 1".to_string(),
                },
                move |ctx: &Arc<Box<TestContext>>| Box::pin(increase_context_value(ctx.clone())),
            )
            .await;

        assert!(context.value.lock().await.eq(&0));
        assert!(registry.trigger("on_event", context.clone()).await.is_ok());
        assert!(context.value.lock().await.eq(&1));
    }

    #[registerable_hook]
    async fn increase_context_value_derive_macro(context: Arc<Box<TestContext>>) -> Result<(), String> {
        let mut guard = context.value.lock().await;
        *guard += 1;

        Ok(())
    }
    #[tokio::test]
    async fn test_hook_registry_registration_with_derive_function() {
        let registry = HookRegistry::new();

        let context = Arc::new(Box::new(TestContext {
            value: Mutex::new(0),
        }));

        registry
            .register(
                "on_event",
                HookMetadata {
                    priority:    0,
                    description: "Increment context value by 1".to_string(),
                },
                increase_context_value_derive_macro::register(),
            )
            .await;

        assert!(context.value.lock().await.eq(&0));
        assert!(registry.trigger("on_event", context.clone()).await.is_ok());
        assert!(context.value.lock().await.eq(&1));
    }
}
