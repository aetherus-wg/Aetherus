use crate::geom::Collide;

pub trait Split<T, P>: Collide<T>
where
    Self: Sized
{
    type Inst;
    /// Split the instance into a list of new instances, discarding intermediate primitives used
    fn split(&self, other: &T) -> Self::Inst;

    /// Split the instance into a list of new instances, and return the list of primitives used to
    /// split it
    fn split_transparent(&self, other: &T) -> (Self::Inst, P);
}
