//! Immutable scientific figures shared by the engine and native renderers.
//! No GUI, interpreter or mathematical-model dependency.
use std::{fmt, sync::Arc};

pub const MAX_POINTS: usize = 100_000;
pub const MAX_EVENTS: usize = 16;
pub const MAX_EXECUTION_POINTS: usize = 200_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphicsError(pub String);
impl fmt::Display for GraphicsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for GraphicsError {}

#[derive(Clone, Debug, PartialEq)]
pub struct PlotStyle {
    pub color: [u8; 3],
    pub width: f32,
}
impl Default for PlotStyle {
    fn default() -> Self {
        Self {
            color: [28, 125, 118],
            width: 2.,
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct AxisConfig {
    pub label: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlotSeries {
    name: String,
    points: Arc<[[f64; 2]]>,
    style: PlotStyle,
}
impl PlotSeries {
    pub fn with_style(mut self, style: PlotStyle) -> Self {
        self.style = style;
        self
    }
    pub fn line(name: impl Into<String>, x: &[f64], y: &[f64]) -> Result<Self, GraphicsError> {
        if x.len() != y.len() {
            return Err(GraphicsError(
                "plot: vetores devem ter o mesmo número de elementos".into(),
            ));
        }
        if x.len() > MAX_POINTS {
            return Err(GraphicsError(
                "plot: limite de 100000 pontos por curva".into(),
            ));
        }
        if x.iter().chain(y).any(|v| !v.is_finite()) {
            return Err(GraphicsError(
                "plot: coordenadas devem ser reais e finitas".into(),
            ));
        }
        let points = x
            .iter()
            .zip(y)
            .map(|(&x, &y)| [x, y])
            .collect::<Vec<_>>()
            .into();
        Ok(Self {
            name: name.into(),
            points,
            style: PlotStyle::default(),
        })
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn points(&self) -> &[[f64; 2]] {
        &self.points
    }
    pub fn style(&self) -> &PlotStyle {
        &self.style
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Figure {
    pub title: String,
    pub x_axis: AxisConfig,
    pub y_axis: AxisConfig,
    series: Vec<PlotSeries>,
}
impl Figure {
    pub fn from_series(
        title: String,
        x_label: String,
        y_label: String,
        series: Vec<PlotSeries>,
    ) -> Self {
        Self {
            title,
            x_axis: AxisConfig { label: x_label },
            y_axis: AxisConfig { label: y_label },
            series,
        }
    }
    pub fn line(
        title: impl Into<String>,
        x_label: impl Into<String>,
        y_label: impl Into<String>,
        series: PlotSeries,
    ) -> Self {
        Self {
            title: title.into(),
            x_axis: AxisConfig {
                label: x_label.into(),
            },
            y_axis: AxisConfig {
                label: y_label.into(),
            },
            series: vec![series],
        }
    }
    pub fn series(&self) -> &[PlotSeries] {
        &self.series
    }
    pub fn point_count(&self) -> usize {
        self.series.iter().map(|s| s.points.len()).sum()
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum PlotEvent {
    Show(Figure),
}
impl PlotEvent {
    pub fn figure(&self) -> &Figure {
        match self {
            Self::Show(figure) => figure,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn snapshots_preserve_precision_order_and_original_vectors() {
        let mut x = vec![3., 1., 2., 2.];
        let y = vec![f64::EPSILON, 2., -3., 4.];
        let series = PlotSeries::line("y", &x, &y).unwrap();
        x[0] = 99.;
        assert_eq!(
            series.points(),
            &[[3., f64::EPSILON], [1., 2.], [2., -3.], [2., 4.]]
        );
        let clone = series.clone();
        assert!(Arc::ptr_eq(&series.points, &clone.points));
        assert_eq!(y, [f64::EPSILON, 2., -3., 4.]);
    }
    #[test]
    fn incompatible_nonfinite_and_oversized_data_are_rejected() {
        assert!(PlotSeries::line("y", &[0.], &[]).is_err());
        assert!(PlotSeries::line("y", &[f64::INFINITY], &[0.]).is_err());
        assert!(PlotSeries::line("y", &[0.], &[f64::NAN]).is_err());
        assert!(
            PlotSeries::line("y", &vec![0.; MAX_POINTS + 1], &vec![0.; MAX_POINTS + 1]).is_err()
        );
        assert!(PlotSeries::line("y", &[], &[]).unwrap().points().is_empty());
    }
}
