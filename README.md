# Evanescence

_An exploration in the visualization of hydrogenic orbitals._

Links: [web application](https://al2me6.github.io/evanescence), [continuous benchmarks](https://al2me6.github.io/evanescence/dev/bench).

_Evanescence_ is a from-the-ground-up implementation of hydrogenic orbitals for the purpose of delivering high-quality, comprehensive, and interactive visualizations. Following prior work by Tully _et al._ [\[1\]](#references), _Evanescence_ provides 3D point-cloud visualizations of orbitals, obtained by Monte Carlo sampling of the orbital's probability distribution.

_Evanescence_ follows a philosophy of _no hard-coding_. That is, computations are always performed using the full generality of the underlying equations. Thus, all implemented computations are available for arbitrary parameters, up to numerical precision limitations.

As _Evanescence_ is built solely as a visualization tool, absolute accuracy or numerical precision are non-goals. Thus, `f32` is used over `f64` and shortcuts that do not significantly affect accuracy (as indicated by unit tests) may be taken where performance demands.

Supported orbital types include:

* Real hydrogen orbitals
* Complex hydrogen orbitals
* Linear combinations of real hydrogen orbitals (hybridization)

Also provided are supplemental visualizations, including:

* Nodal (radial, angular) surfaces
* Wavefunction and probability density cross-sections
* Radial wavefunction and radial probability distribution curves
* Probability density isosurfaces

## References

1. Tully, S. P.; Stitt, T. M.; Caldwell, R. D.; Hardock, B. J.; Hanson, R. M.; Maslak, P. Interactive Web-Based Pointillist Visualization of Hydrogenic Orbitals Using Jmol. _Journal of Chemical Education_ **2013**, _90_ (1), 129-131. DOI: [10.1021/ed300393s](https://doi.org/10.1021/ed300393s).

## License

This project is released under the GNU Affero GPL license, version 3.
