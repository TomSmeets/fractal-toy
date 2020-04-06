use std::cmp::Ordering;
use std::iter::Peekable;

pub enum ComparedValue<L, R> {
    Left(L),
    Right(R),
    Both(L, R),
}

pub struct CompareIter<I, J, FCmp>
where
    I: Iterator,
    J: Iterator,
    FCmp: FnMut(&I::Item, &J::Item) -> Ordering,
{
    left: Peekable<I>,
    right: Peekable<J>,
    fcmp: FCmp,
}

impl<I, J, FCmp> CompareIter<I, J, FCmp>
where
    I: Iterator,
    J: Iterator,
    FCmp: FnMut(&I::Item, &J::Item) -> Ordering,
{
    pub fn new(left: I, right: J, fcmp: FCmp) -> Self {
        CompareIter {
            left: left.peekable(),
            right: right.peekable(),
            fcmp,
        }
    }
}

impl<I, J, FCmp> Iterator for CompareIter<I, J, FCmp>
where
    I: Iterator,
    J: Iterator,
    FCmp: FnMut(&I::Item, &J::Item) -> Ordering,
{
    type Item = ComparedValue<I::Item, J::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        let ord = match (self.left.peek(), self.right.peek()) {
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (None, None) => return None,
            (Some(old), Some(new)) => (self.fcmp)(old, new),
        };

        Some(match ord {
            Ordering::Less => ComparedValue::Left(self.left.next().unwrap()),
            Ordering::Equal => {
                ComparedValue::Both(self.left.next().unwrap(), self.right.next().unwrap())
            },
            Ordering::Greater => ComparedValue::Right(self.right.next().unwrap()),
        })
    }
}
