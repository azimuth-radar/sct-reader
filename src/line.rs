use crate::{
    colour::Colour,
    position::{Position, Valid},
};

pub trait Line {
    fn start(&self) -> Position<Valid>;
    fn end(&self) -> Position<Valid>;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SimpleLine {
    pub start: Position<Valid>,
    pub end: Position<Valid>,
}
impl SimpleLine {
    pub fn new(start: Position<Valid>, end: Position<Valid>) -> SimpleLine {
        SimpleLine { start, end }
    }
}
impl Line for SimpleLine {
    fn start(&self) -> Position<Valid> {
        self.start
    }
    fn end(&self) -> Position<Valid> {
        self.end
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColouredLine {
    pub line: SimpleLine,
    pub colour: Option<Colour>,
}
impl ColouredLine {
    pub fn new(
        start: Position<Valid>,
        end: Position<Valid>,
        colour: Option<Colour>,
    ) -> ColouredLine {
        ColouredLine {
            line: SimpleLine::new(start, end),
            colour,
        }
    }
    pub fn colour(&self) -> Option<Colour> {
        self.colour
    }
}
impl Line for ColouredLine {
    fn start(&self) -> Position<Valid> {
        self.line.start
    }
    fn end(&self) -> Position<Valid> {
        self.line.end
    }
}

#[derive(Debug)]
pub struct LineGroup<L: Line> {
    pub name: String,
    pub lines: Vec<L>,
}
impl<L: Line> LineGroup<L> {
    pub fn new(name: String, lines: Vec<L>) -> LineGroup<L> {
        LineGroup { name, lines }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
}
