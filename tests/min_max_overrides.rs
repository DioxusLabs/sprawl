#[cfg(test)]
mod min_max_overrides {

    use taffy::prelude::*;

    #[test]
    fn min_overrides_max() {
        let mut taffy = Taffy::new();

        let child = taffy.new_leaf(Style {
            size: Size { width: Dimension::Length(50.0), height: Dimension::Length(50.0) },
            min_size: Size { width: Dimension::Length(100.0), height: Dimension::Length(100.0) },
            max_size: Size { width: Dimension::Length(10.0), height: Dimension::Length(10.0) },
            ..Default::default()
        });

        taffy.compute_layout(
            child,
            Size { width: AvailableSpace::Definite(100.0), height: AvailableSpace::Definite(100.0) },
        );

        assert_eq!(taffy.layout(child).size, Size { width: 100.0, height: 100.0 });
    }

    #[test]
    fn max_overrides_size() {
        let mut taffy = Taffy::new();

        let child = taffy.new_leaf(Style {
            size: Size { width: Dimension::Length(50.0), height: Dimension::Length(50.0) },
            max_size: Size { width: Dimension::Length(10.0), height: Dimension::Length(10.0) },
            ..Default::default()
        });

        taffy.compute_layout(
            child,
            Size { width: AvailableSpace::Definite(100.0), height: AvailableSpace::Definite(100.0) },
        );

        assert_eq!(taffy.layout(child).size, Size { width: 10.0, height: 10.0 });
    }

    #[test]
    fn min_overrides_size() {
        let mut taffy = Taffy::new();

        let child = taffy.new_leaf(Style {
            size: Size { width: Dimension::Length(50.0), height: Dimension::Length(50.0) },
            min_size: Size { width: Dimension::Length(100.0), height: Dimension::Length(100.0) },
            ..Default::default()
        });

        taffy.compute_layout(
            child,
            Size { width: AvailableSpace::Definite(100.0), height: AvailableSpace::Definite(100.0) },
        );

        assert_eq!(taffy.layout(child).size, Size { width: 100.0, height: 100.0 });
    }
}
