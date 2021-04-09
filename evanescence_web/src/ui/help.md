<!-- This file is rendered at build time into HTML and included in the binary as the help page. -->

## Using Evanescence

For the best experience, please view this page on a computer.

### Visualization modes

You can select different visualization modes using the tab bar:

![Tab bar](img/tab-bar.png)

Each mode offers a different type of orbital to visualize.

* "Real (Simple)" provides a selection of common atomic orbitals for quick access.
* "Real (Full)" allows arbitrary quantum numbers (up to *n* = 8) to be specified.
* "Complex" displays the complex orbitals, which are useful in certain contexts.
* "Hybrid" provides visualizations for hybridized orbitals and their symmetry.

### Help for specific features

Text with <span class="tooltip">dotted underlines<span class="description">Like this!</span></span> have explanations. Hover over them with your cursor to read them.

#### What does the main visualization display?

The main visualization on the left side (or top) of your screen is a point cloud that simulates what you might find if you tried to determine the location of an electron in that orbital a large number of times. The denser and larger the points are, the more likely it is that the electron will be found at that location.

The size of the point corresponds to the magnitude of the wavefunction at that point. In the Real and Hybrid modes, the color of the point corresponds to the value of the wavefunction – red is large (relatively speaking) and positive, blue is large and negative, white is near zero. In Complex mode, color corresponds to complex phase.

For more information, see <a href="https://doi.org/10.1021/ed300393s" target="_blank">Tully et al. 2013</a>.

#### Surface plots are ragged or "pointy"

You need more samples to adequately reflect the geometry of the orbital – increase the quality.

#### 3D isosurface visualization is blank

If you see a blank isosurface plot, try increasing the quality – the orbital may be too intricate to be sufficiently well sampled at the current quality.

### Interacting with visualizations

For 3D visualizations, you can:

* Left-click and drag to rotate
* Right-click and drag to pan
* Scroll to zoom

Additional options are available in the toolbar that appears when you hover over a plot. For example, you can use the "camera" tool to save a screenshot of the plot.

## About Evanescence

&copy; Alvin Meng 2021

Built in <a href="https://rust-lang.org" target="_blank">Rust</a> and WebAssembly using the <a href="https://yew.rs" target="_blank">Yew</a> framework and the <a href="https://plotly.com/javascript" target="_blank">Plotly.js</a> graphing library.

This software is released under the <a href="https://www.gnu.org/licenses/agpl-3.0.en.html" target="_blank">GNU Affero GPL v3</a> license. The source code can be found on <a href="https://github.com/al2me6/evanescence" target="_blank">GitHub</a>.
