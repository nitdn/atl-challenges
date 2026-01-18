use raylib::prelude::*;
pub type Point = Vector2;
pub type BBox = Rectangle;
type NodeId = u16;

pub const MAX_QUADTREE_DEPTH: u16 = 64;

/// Finds the midpoint between two points
pub fn middle(&point1: &Point, &point2: &Point) -> Point {
    (point1 + point2) * 0.5
}
/// Anything that can be a 2D `BBox`
pub trait BBoxExtension {
    fn minimum(&self) -> Point;
    fn maximum(&self) -> Point;

    /// Extends the bbox in a not insane way
    fn extend(&self, point: &Point) -> Self;

    /// New bbox with all the points bounded in it
    fn new_bounded(points: &[Point]) -> Self
    where
        Self: Default,
    {
        points
            .iter()
            .fold(Self::default(), |bbox, point| bbox.extend(point))
    }
}

impl BBoxExtension for BBox {
    fn minimum(&self) -> Point {
        Point::new(self.x, self.y)
    }
    fn maximum(&self) -> Point {
        Point::new(self.width, self.height) + self.minimum()
    }

    fn extend(&self, point: &Point) -> Self {
        let x = self.x.min(point.x);
        let y = self.y.min(point.y);
        let max_x = f32::max(self.x + self.width, point.x);
        let max_y = f32::max(self.y + self.height, point.y);
        let width = max_x - x;
        let height = max_y - y;
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

#[derive(Default, Debug)]
pub struct Node {
    children: [Option<NodeId>; 4],
}

#[derive(Default, Debug)]
pub struct QuadTree {
    bbox: BBox,
    root: Option<NodeId>,
    nodes: Vec<Node>,
    points: Vec<Point>,
    node_points_begin: Vec<NodeId>,
}

impl QuadTree {
    pub fn build(points: &[Point]) -> Self {
        let mut result = Self::default();
        result.points.extend_from_slice(points);
        let bbox = result.bbox;
        result.root = result.build_impl(bbox, points, MAX_QUADTREE_DEPTH);
        result.node_points_begin.push(result.points.len() as NodeId);
        result
    }

    fn build_impl(&mut self, bbox: BBox, points: &[Point], depth: u16) -> Option<NodeId> {
        if points.is_empty() {
            return None;
        }
        let result = self.nodes.len() as NodeId;
        self.nodes.push(Default::default());
        if points.len() == 1 || depth == 0 {
            return Some(result);
        }

        let center = middle(&bbox.minimum(), &bbox.maximum());
        let (top, bottom): (Vec<_>, Vec<_>) = points.iter().partition(|&point| point.y < center.y);

        let (top_left, top_right): (Vec<_>, Vec<_>) =
            top.iter().partition(|&point: &&Point| point.x > center.x);
        let (bottom_left, bottom_right): (Vec<_>, Vec<_>) = bottom
            .iter()
            .partition(|&point: &&Point| point.x > center.x);

        self.nodes[result as usize].children[0] = self.build_impl(
            BBox::new_bounded(&[bbox.minimum(), center]),
            &bottom_left,
            depth - 1,
        );
        self.nodes[result as usize].children[1] = self.build_impl(
            BBox::new_bounded(&[bbox.minimum(), center]),
            &bottom_right,
            depth - 1,
        );
        self.nodes[result as usize].children[2] = self.build_impl(
            BBox::new_bounded(&[bbox.minimum(), center]),
            &top_left,
            depth - 1,
        );
        self.nodes[result as usize].children[3] = self.build_impl(
            BBox::new_bounded(&[bbox.minimum(), center]),
            &top_right,
            depth - 1,
        );
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_bboxes() {
        let bbox = BBox::new(12.0, 12.0, 12.0, 12.0);
        assert_eq!(bbox.maximum(), Point::new(24.0, 24.0));
        let p1 = dbg!(Point::new(1.0, 13.0));
        let bbox = dbg!(bbox.extend(&p1));
        assert_eq!(bbox, BBox::new(1.0, 12.0, 12.0, 13.0));
        let p1 = dbg!(Point::new(13.0, 25.0));
        let bbox = dbg!(bbox.extend(&p1));
        assert_eq!(bbox.x, 1.0);
        assert_eq!(bbox.y, 12.0);
        panic!()
    }
}
