use spacetimedb::ReducerContext;

pub trait FromCtx<T>: Sized {
    /// Converts to this type from the input type.
    fn from_ctx(ctx: &ReducerContext, value: T) -> Self;
}
