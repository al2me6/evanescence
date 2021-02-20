# Version History

## 0.4.4, 2021-02-20

### Features

* The handling of hybrid orbitals has been revamped. It is now possible to show silhouettes of all hybrid orbitals of a certain kind at once to illustrate symmetry.
* More detailed information is provided about hybrid orbitals.
* The coloring of isosurfaces has been improved.

### Known Issues

* Sometimes the "Show symmetry" box in hybrid mode becomes mysteriously checked if the option is enabled, a different mode is selected, and hybrid mode is activated again.

## 0.4.3, 2021-02-19

### Features

* Plots' background colors are no longer transparent, such that useful screenshots can be exported using the built-in "download plot" function.
* Nodal surfaces are now only computed when they are known to exist, improving performance when nodal surface display is enabled but there is no actual node to be shown.
* The user interface now uses transitions.
* The "help" cursor is now displayed when hovering over text that has a tooltip.
* Mode selection now uses a tab bar instead of a dropdown menu to improve clarity and discoverability.

### Changes

* Viewport controls are now hidden by default.

## 0.4.2, 2021-02-17

### Features

* Basic plotting of hybridized orbitals is now implemented.
* The color scale is now labeled more clearly in Complex mode.

## 0.4.1, 2021-02-17

### Bugfixes

* Fix rendering of angular nodes and cross-section indicators. These were regressions introduced in version 0.4.0.

## 0.4.0, 2021-02-12

### Features

* Plotting of complex orbitals is now implemented. Bask in ~~rainbow~~ unicorn vomit glory!
* The state management logic has been revamped, yet again. This opens the way to the plotting of hybridized and molecular orbitals in the future.

## 0.3.8, 2021-02-11

### Bugfixes

* Fix depth buffer artifacts in the main pointillist by setting point opacity to 1.0. This resolves the issue of points disappearing while overlapping other lobes.
* Correct accidental use of `ψ` instead of `R` for the radial wavefunction.

## 0.3.7, 2021-02-05

### Features

* Supplemental plots now have descriptions.
* Annotate axes with the cooresponding functions (ex. `r`, `r^2R^2`) where necessary.

### Bugfixes

* Supplemental plots are now resized appropriately when the size of controls and text changes.

## 0.3.6, 2021-02-04

### Features

* Hover tooltips containing explanations for configuration options are now displayed.

### Known Issues

* Tooltips may be cut off by the screen edge on certain screen geometries.

## 0.3.5, 2021-02-02

### Features

* Presets for common quantum number sets. The full quantum number selector can be enabled by selecting "Custom" from the preset dropdown.
* Display the corresponding subshell and orbital names in dropdown menus, if applicable and possible.

## 0.3.4, 2021-01-28

### Features

* Blobs! Initial implementation of 3D isosurface plots.
* Add favicon and `meta` description tags.

### Changes

* Allow pan and zoom for radial plots.

### Known Issues

* The viewport for supplemental visualizations sometimes resets mysteriously.
* Isosurface cutoffs are in need of fine-tuning.

## 0.3.3, 2021-01-24

### Bugfixes

* Remove extraneous hover highlighting on cross-section indicators.

## 0.3.2, 2021-01-24

### Features

* A contour line is always placed at zero on cross-section plots. i.e., the nodes are always highlighted.

### Changes

* Values near zero are now shown in white instead of yellow on cross-section plots.

## 0.3.1, 2021-01-23

### Bugfixes

* Correctly remove the cross-section indicator from the main plot when the cross-section is turned off.

## 0.3.0, 2021-01-23

### Features

* Implement cross-sectional views. This visualization plots the xy-, yz-, or xz-plane cross section of a wavefunction as a 3D contour, clarifying the features of the orbital on that specific plane. An indicator is drawn on the pointillist visualization to show the plane on which the cross-section is taken.

## 0.2.1, 2021-01-23

### Bugfixes

* Fix display of negative numbers in dropdown menus. This is a regression introduced in v0.2.0.

## 0.2.0, 2021-01-22

### Features

* Initial implementation of supplemental visualizations. The first batch includes three radial plots: The radial wavefunction `R`, the radial probability `R^2`, and the radial probability distribution `r^2 R^2`.

### Bugfixes

* Mitigate artifacting on orbitals with `l` = 4, `m` = ±4.
* Display quality names in sentence case.
* Always sample the origin in nodal surface plots. This improves the accuracy of certain features.

## 0.1.1, 2021-01-21

### Changes

* Improve dark theme appearance.
* Switch to Lato 2.0 font, hosted by Adobe Fonts.

## 0.1.0, 2021-01-21

### Features

* Improve orbital information panel, including giving the number of radial and angular nodes.

## 0.0.5, 2021-01-20

### Features

* Dark mode and page styling.
* Additional compilation flags for release builds.

## 0.0.4, 2021-01-19

### Features

* Third implementation of pointillist plot trace management. This brings significant speed improvements when switching quantum numbers or quality with nodal surfaces shown, by reducing the number of API calls and trace deletions/additions.

### Changes

* Reduce default quality from Medium to Low, raising the point size slightly to compensate.

## 0.0.3, 2021-01-18

### Features

* Compute radial and angular nodes separately. This allows the kind(s) of nodes being plotted to be selected, while also reducing the number of artifacts by reducing the number of intersections plotted by the same trace.
  * The management of traces in the pointillist plot is revamped to enable this functionality.

### Bugfixes

* Set the plot range and aspect ratio of plots at all times to eliminate jumps resulting from automatic plot range determination.

## 0.0.2, 2021-01-17

### Features

* (Rudimentary) plotting of nodal surfaces.
* Internal improvements to state handling.

## 0.0.1, 2021-01-16

### Features

* Initial implementation of the pointillist visualization, including the selection of quantum numbers and quality presets.
* Set up continuous integration for automatic deployment of the website.
