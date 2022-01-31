# Version History

## 0.4.17, 2022-01-31

### Features

* Improved display of linear combinations in Hybrid mode.
* In Real mode, nodal surfaces are no longer unnecessarily rerendered when changing quality.

### Internal improvements

* Migrated to Yew v0.19.

## 0.4.16, 2021-11-05

### Features

* Supplemental plots can now be maximized for easier viewing.

## 0.4.15, 2021-10-22

### Features

* Nodal surfaces are now plotted exactly in Real mode by numerically solving for their positions and drawing spheres, cones, and planes. This *significantly* improves their appearance and accuracy, compared to the previous approach of drawing them as isosurfaces of the radial and angular wavefunctions.

### Changes

* Expressions are now given for *h* orbitals.
* The maximum `n` value has been increased from 8 to 12 in Real and Complex modes.

## 0.4.14, 2021-10-12

### Features

* Added an optional 'apply' button to the quantum number selectors in Real and Complex modes. This is to help reduce the number of pauses experienced while changing multiple quantum number dropdowns, whose options are interlinked. Thanks to Tyler A. for the suggestion!

## 0.4.13, 2021-10-04

### Features

* Basic plotting of the molecular orbitals of the hydrogen molecule-ion is now implemented.

### Fixes

* State is now cleared on crash, so in the (rare) scenario of a crash reloading will not result in an immediate crash with the same error.

## 0.4.12, 2021-09-20

### Features

* The quantum numbers of the orbital are now displayed in Real (Simple) mode.

### Changes

* The sampling range for real and complex orbitals is now determined using numerical integration instead of a 'heuristic' function. More specifically, it is the dimension of the sphere that encompasses ~99.8% of all probability density.
* Radial probability distribution curves are now validated by checking that they sum to unity. This is tested for all orbitals with n <= 8 to be true to within 0.005 and is also displayed in the console. Note that the values come out slightly less than one, since we only sample a finite (albeit the most important) portion of the curve.
* The opacity of angular nodes has been increased slightly to aid visibility.

## 0.4.11, 2021-05-04

### Features

* Internal improvements to the handling of hybrid orbitals.
* sp<sup>3</sup>d and sp<sup>3</sup>d<sup>2</sup> orbitals are now available.

### Changes

* The number of estimation samples used for determining the maximum probability density attained by an orbital has been increased, improving sample accuracy in certain situations.
* The border around the main plot has been removed for a more immersive experience.

## 0.4.10, 2021-05-02

### Features

* Cross-section probability density plots are now available in all modes.

## 0.4.9, 2021-04-27

### Features

* State is now persisted across reloads.
* Cross-sections are now available in Complex mode.
* Nodal surfaces and 3D isosurfaces are now available in Hybrid mode.

### Changes

* The help page has been expanded and reorganized.
* All links now open in a new tab.
* Orbital names are correctly italicized where possible.
* Minor styling improvements.

### Bugfixes

* There are no longer color artifacts in complex cross-section plots if all values are zero.

## 0.4.8, 2021-04-13

### Features

* Cross-section plots are now available in Complex mode.
* Plot updates are now timed, and the times are printed to console.
* The z-axis label for cross-section plots now clearly indicates which variable is zero.
* A message is now displayed if Evanescence crashes to notify the user to refresh the page.

### Changes

* The help page has been expanded.
* Minor improvements to help tooltips.
* The font size of the z-axis label for cross-section plots has been reduced.
* Symbols for quantum numbers are now italicized where possible.

### Bugfixes

* Silhouettes in Hybrid mode are no longer cropped. This was a regression introduced in version 0.4.7.
* The example tooltip on the help page now works again.

### Known issues

* Complex cross-section plots tend to have more significant numerical instability artifacts than other cross-section plots. This is due to the nature of modulus plots and cannot be resolved.

## 0.4.7, 2021-03-11

### Features

* The isosurface cutoff value is now shown for 3D isosurface plots.

### Changes

* Pointillist plots are now based on probability density instead of wavefunction value. This is more correct and produces higher-quality, more well-defined plots. Unfortunately, this also has a rather significant perform cost that can only be partially ameliorated.
* Quality values have been adjusted to improve performance, since fewer points are now needed to achieve the same visual quality.
* Various descriptions have been clarified and made more rigorous.
* Point sizes have been adjusted to emphasize the outer lobes of *s* orbitals.
* Nodal surfaces are now more opaque and clearer.
* Removed the "radial probability density" plot. This plot is somewhat confusing and does not have a signifcant use when "radial probability distribution" is also provided.
  * The probability distribution curve is now shaded to reflect that its integral corresponds to probability.

## 0.4.6, 2021-03-02

### Bugfixes

* Fixed styling of help page title on iOS.
* Fixed styling of tab bar on iOS.
* Do not scroll the page itself when the help window is open on mobile devices.
  * Note that this is [known to be (intentionally) broken](https://bugs.webkit.org/show_bug.cgi?id=153852#c34) on iOS when the navigation bar is collapsed.
* Always set correct page height on mobile devices (this is now done in Rust by setting the height to `window.innerHeight`, or the space available after the navigation bars have been accounted for).
* Do not zoom in when using selectors on iOS.
* Correctly update layout on orientation change.

## 0.4.5, 2021-03-01

### Features

* There is now a brief help page, accessed through the "?" button in the top-right corner.

## 0.4.4, 2021-02-20

### Features

* The handling of hybrid orbitals has been revamped. It is now possible to show silhouettes of all hybrid orbitals of a certain kind to illustrate symmetry.
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

* Plot controls are now hidden by default.

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
