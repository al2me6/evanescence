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
* "MO" displays orbitals of the H<sub>2</sub><sup>+</sup> molecule-ion.

### Help for specific features

Text with <span class="tooltip">dotted underlines<span class="description">Like this!</span></span> have explanations. Hover over them with your cursor to read them.

### Interacting with visualizations

For 3D visualizations, you can:

* Left-click and drag to rotate
* Right-click and drag to pan
* Scroll to zoom

Additional options are available in the toolbar that appears when you hover over a plot. For example, you can use the "home" button to reset the viewport position or the "camera" button to save a screenshot of the plot.

### What are the units?

All plots are in terms of [Bohr radii](https://en.wikipedia.org/wiki/Bohr_radius), which is roughly equivalent the radius of the ground-state orbit of the hydrogen atom in the classical Bohr model. One Bohr radius, symbol *a*<sub>0</sub>, is approximately 52.9 pm.

### What does the main visualization display?

The main visualization on the left side (or top) of your screen is a point cloud that simulates what you might find if you tried to determine the location of an electron in that orbital a large number of times. The denser and larger the points are, the more likely it is that the electron will be found at that location.

The size of the point corresponds to the magnitude of the wavefunction at that point. In the Real, Hybrid, and MO modes, the color of the point corresponds to the value of the wavefunction – red is large (relatively speaking) and positive, blue is large and negative, white is near zero. In Complex mode, color corresponds to [complex phase](https://en.wikipedia.org/wiki/Argument_(complex_analysis)).

### Common problems

#### Ragged or "pointy" cross-section plot; blank 3D isosurface

You need more samples to adequately reflect the geometry of the orbital – increase the quality.

#### Jumbled triangles appearing on angular nodes

This is caused by numerical instabilities in the graphing library. Try changing the quality.

#### Points in the point cloud appear very large

This occurs on certain browser-operating system combinations, specifically Firefox on Unix systems and Safari on macOS. Please use a different browser.

## References & Further Reading

* Tully *et al.* 2013, [Interactive Web-Based Pointillist Visualization of Hydrogenic Orbitals Using Jmol](https://doi.org/10.1021/ed300393s)
* Douglas 1990, [Visualization of Electron Clouds in Atoms and Molecules](https://doi.org/10.1021/ed067p42)
* Ogryzlo and Porter 1963, [Contour Surfaces for Atomic and Molecular Orbitals](https://doi.org/10.1021/ed040p256)
* Mark Winter, [The Orbitron](https://winter.group.shef.ac.uk/orbitron/)
* Paul Falstad, [Hydrogen Atom Orbital Viewer](https://www.falstad.com/qmatom/)
* Paul Falstad, [Molecular Orbital Viewer](https://www.falstad.com/qmmo/)

## Acknowledgements

I'd like to extend my gratitude to the following people for helping this project come true:

* Dr. B. Kennedy, for overseeing the initial phases of this project;
* Mr. H. Kauffman, for inspiring me to take on this project;
* Dr. J. Osborne, for giving me invaluable advice and pointers;
* Michelle R., for putting up with my barrage of progress screenshots with great enthusiasm;
* Ms. A. Holman, Liam C., and Tyler A., for offering helpful feedback; and
* Thomas L., for suggesting that I look into a project in computational chemistry.

## About Evanescence

&copy; [Alvin Q. Meng](mailto:alvin.q.meng@gmail.com) 2020-2021

Built in [Rust](https://rust-lang.org) and WebAssembly using the [Yew](https://yew.rs) framework and the [Plotly.js](https://plotly.com/javascript) graphing library.

This software is released under the [GNU Affero GPL v3](https://www.gnu.org/licenses/agpl-3.0.en.html) license. The source code can be found on [GitHub](https://github.com/al2me6/evanescence).
