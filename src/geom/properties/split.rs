use crate::geom::Collide;

pub trait Split<T>: Collide<T>
where
    Self: Sized
{
    fn split(&self, other: &T) -> Vec<Self>;
}
