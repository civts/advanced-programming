window.SIDEBAR_ITEMS = {"enum":[["LabelAreaPosition","Specifies one of the four label positions around the figure."],["SeriesLabelPosition","Useful to specify the position of the series label."]],"struct":[["ChartBuilder","The helper object to create a chart context, which is used for the high-level figure drawing."],["ChartContext","The context of the chart. This is the core object of Plotters."],["ChartState","A chart context state - This is the data that is needed to reconstruct the chart context without actually drawing the chart. This is useful when we want to do realtime rendering and want to incrementally update the chart."],["DualCoordChartContext","The chart context that has two coordinate system attached. This situation is quite common, for example, we with two different coodinate system. For instance this example  This is done by attaching  a second coordinate system to ChartContext by method ChartContext::set_secondary_coord. For instance of dual coordinate charts, see this example. Note: `DualCoordChartContext` is always deref to the chart context."],["DualCoordChartState","The chart state for a dual coord chart, see the detailed description for `ChartState` for more information about the purpose of a chart state. Similar to ChartState, but used for the dual coordinate charts."],["MeshStyle","The struct that is used for tracking the configuration of a mesh of any chart"],["SecondaryMeshStyle","The style used to describe the mesh and axis for a secondary coordinate system."],["SeriesAnno","The annotations (such as the label of the series, the legend element, etc) When a series is drawn onto a drawing area, an series annotation object is created and a mutable reference is returned."],["SeriesLabelStyle","The struct to specify the series label of a target chart context"]]};