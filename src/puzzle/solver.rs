//! Solver for packing problems.
use std::fmt::{Display, Formatter, Error};

use super::vector::{VectorAdd, VectorDifference};
use super::piece::{MinimumPosition, Position, Positionable, Translatable, Transformable, Normalizable, Piece};
use super::pieces::Bag;

/// Region to be packed.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Target<T> {
    collection: Vec<Position<T>>,
}

impl<T> Target<T> where T: PartialOrd + Ord + PartialEq + Eq + Clone {
    /// Create a new `Target` from a collection of `Position`s.
    pub fn new(collection: Vec<Position<T>>) -> Target<T> {
        Target { collection }
    }

    /// Determine if there is nothing left to pack.
    pub fn is_packed(&self) -> bool {
        self.collection.is_empty()
    }

    /// Determine if a `Piece` can be placed in the `Target`.
    pub fn fits(&self, piece: &Piece<T>) -> bool {
        piece.iter().all(|position| self.collection.contains(&position))
    }

    /// Place a `Piece` in the `Target`. *Note* caller is responsible to check
    /// if the `Piece` will actually fit.
    pub fn place(&self, piece: &Piece<T>) -> Target<T> {
        let collection: Vec<Position<T>> = self.collection
            .iter()
            .filter(|position| !piece.contains(position))
            .cloned()
            .collect();

        Target::new(collection)
    }
}

impl<T> MinimumPosition<T> for Target<T> where T: PartialOrd + Ord + Clone {
    fn minimum_position(&self) -> Option<Position<T>> {
        self.collection.iter().min().cloned()
    }
}

/// (Partial) solution of a packing problem. Piece at their correct location are listed.
#[derive(Debug)]
pub struct Solution<T> {
    pieces: Vec<Piece<T>>
}

impl<T> Solution<T> where T : Clone {
    /// Empty solution. Serves as a starting point for the `solve` method.
    pub fn empty() -> Solution<T> {
        Solution { pieces: vec!() }
    }

    /// Record a `Piece` as part of the `Solution`.
    ///
    /// Returns a new `Solutions` with the `Piece` added. *Note* the caller is
    /// responsible for checking if the `Piece` actually fits in the `Target`.
    pub fn record(&self, piece: &Piece<T>) -> Solution<T> {
        let mut pieces: Vec<Piece<T>> = self.pieces.to_vec();
        pieces.push(piece.clone());

        Solution { pieces }
    }
}

impl Display for Solution<(i8, i8, i8)> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "<")?;
        for piece in &self.pieces {
            write!(f, "{}", piece)?;
        }
        write!(f, ">")
    }
}

impl Display for Solution<(i8, i8)> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "<")?;
        for piece in &self.pieces {
            write!(f, "{}", piece)?;
        }
        write!(f, ">")
    }
}


/// Attempt to pack all the `Piece`s in the `Bag` into the `Target` region. When
/// a solution is found, the `when_solved` callback is called with that solution.
pub fn solve<F, T>(target: &Target<T>, bag: Bag<T>, when_solved: &mut F) where F: (FnMut(Solution<T>)) + Sized, T: Clone + PartialOrd + Ord + Transformable + Normalizable<T> + VectorDifference<T> + VectorAdd<T> {
    let partial_solution: Solution<T> = Solution::empty();
    solve_with(target, bag, partial_solution, when_solved)
}


/// Variant of the `solve` method that allows for a different starting point.
pub fn solve_with<F, T>(target: &Target<T>, bag: Bag<T>, partial_solution: Solution<T>, when_solved: &mut F) where F: (FnMut(Solution<T>)) + Sized, T: Clone + PartialOrd + Ord + Transformable + Normalizable<T> + VectorDifference<T> + VectorAdd<T> {
    if target.is_packed() {
        when_solved(partial_solution)
    } else {
        let open_position = target.minimum_position().unwrap();
        for (template, rest_of_bag) in bag {
            for mut piece in template {
                let block = piece.minimum_position().unwrap();
                let translation = block.to(&open_position);
                piece.translate(&translation);
                if target.fits(&piece) {
                    let remaining_target = target.place(&piece);
                    let candidate_solution = partial_solution.record(&piece);
                    solve_with(&remaining_target, rest_of_bag.clone(), candidate_solution, when_solved)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Write;
    use super::super::piece::{Position, Piece, Template};
    use super::super::pieces::Bag;
    use super::*;

    #[test]
    fn piece_should_fit_in_target() {
        let target = Target::new(vec!(
            Position::new(0, 0, 0),
            Position::new(1, 0, 0),
            Position::new(0, 1, 0),
            Position::new(1, 1, 0),
            Position::new(0, 0, 1),
            Position::new(1, 0, 1),
            Position::new(0, 1, 1),
            Position::new(1, 1, 1),
        ));

        let piece = Piece::new(vec!(
            Position::new(0, 0, 0),
            Position::new(1, 0, 0),
            Position::new(0, 1, 0),
            Position::new(0, 0, 1),
        ));

        assert!(target.fits(&piece));
    }

    #[test]
    fn solve_should_pack_pieces() {
        let target = Target::new(vec!(
            Position::new(0, 0, 0),
            Position::new(1, 0, 0),
            Position::new(0, 1, 0),
            Position::new(1, 1, 0),
            Position::new(0, 0, 1),
            Position::new(1, 0, 1),
            Position::new(0, 1, 1),
            Position::new(1, 1, 1),
        ));

        let bag = Bag::new(vec!(
            (2,Template::new(vec!(
                Position::new(0, 0, 0),
                Position::new(1, 0, 0),
                Position::new(0, 1, 0),
                Position::new(0, 0, 1),
            ))),
        ));

        let mut solutions: Vec<Solution<(i8, i8, i8)>> = vec!();
        solve(&target, bag, &mut |solution|{ solutions.push(solution)});
        assert_eq!(solutions.len(), 4);
    }

    #[test]
    fn solutions_should_display_nicely() {
        let solution =
            Solution::empty()
            .record(
                &Piece::new(vec!(
                    Position::new(0, 0, 0),
                    Position::new(1, 0, 0),
                    Position::new(0, 1, 0),
                    Position::new(0, 0, 1),
                )))
            .record(
                &Piece::new(vec!(
                    Position::new(1, 1, 1),
                    Position::new(0, 1, 1),
                    Position::new(1, 0, 1),
                    Position::new(1, 1, 0),
                )));

        let mut output: String = String::new();
        write!(&mut output, "{}", solution).expect("to cleanly write solution");

        assert_eq!(output, String::from("<[(0, 0, 0)(0, 0, 1)(0, 1, 0)(1, 0, 0)][(0, 1, 1)(1, 0, 1)(1, 1, 0)(1, 1, 1)]>"));
    }
}
