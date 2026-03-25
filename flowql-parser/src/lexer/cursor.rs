use std::str::CharIndices;

#[derive(Debug, Clone)]
struct Lookahead<I: Iterator> {
    iter: I,
    buffer: [Option<I::Item>; 2],
}

impl<I: Iterator> Lookahead<I> {
    pub fn new(mut iter: I) -> Self {
        let first = iter.next();
        let second = iter.next();

        Self {
            iter,
            buffer: [first, second],
        }
    }

    pub fn peek(&self) -> Option<&I::Item> {
        self.buffer[0].as_ref()
    }

    pub fn peek_2(&self) -> Option<&I::Item> {
        self.buffer[1].as_ref()
    }
}

impl<I: Iterator> Iterator for Lookahead<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let second = std::mem::replace(&mut self.buffer[1], self.iter.next());
        std::mem::replace(&mut self.buffer[0], second)
    }
}

#[derive(Debug, Clone)]
pub struct SourceReader<'a> {
    iter: Lookahead<CharIndices<'a>>,
    pos: usize,
}

impl<'a> SourceReader<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            iter: Lookahead::new(input.char_indices()),
            pos: 0,
        }
    }

    pub fn next(&mut self) -> Option<char> {
        let (i, next) = self.iter.next()?;
        self.pos = i + next.len_utf8();
        Some(next)
    }

    pub fn peek(&self) -> Option<&char> {
        let (_, next) = self.iter.peek()?;
        Some(next)
    }

    pub fn peek_2(&self) -> Option<&char> {
        let (_, next) = self.iter.peek_2()?;
        Some(next)
    }

    pub fn pos(&self) -> usize {
        self.pos
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookahead() {
        let mut iter = Lookahead::new([4, 9, 16].into_iter());

        assert_eq!(iter.peek(), Some(&4));
        assert_eq!(iter.peek_2(), Some(&9));
        assert_eq!(iter.next(), Some(4));

        assert_eq!(iter.peek(), Some(&9));
        assert_eq!(iter.peek_2(), Some(&16));
        assert_eq!(iter.next(), Some(9));

        assert_eq!(iter.peek(), Some(&16));
        assert_eq!(iter.peek_2(), None);
        assert_eq!(iter.next(), Some(16));

        assert_eq!(iter.peek(), None);
        assert_eq!(iter.peek_2(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn source_reader() {
        let mut reader = SourceReader::new("A🟥ç\n");

        assert_eq!(reader.pos(), 0);
        reader.next();
        assert_eq!(reader.pos(), 1);
        reader.next();
        assert_eq!(reader.pos(), 5);
        reader.next();
        assert_eq!(reader.pos(), 7);
        reader.next();
        assert_eq!(reader.pos(), 8);
        reader.next();
        assert_eq!(reader.pos(), 8);
    }
}
