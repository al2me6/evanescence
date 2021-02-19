macro_rules! description_items {
    ($($name:ident),+ $(,)?) => {
        #[allow(dead_code)]
        pub(crate) struct Descriptions {
            $(pub(crate) $name: &'static str),+
        }
    }
}
description_items!(
    qn_dropdown,
    supplement,
    rad_nodes,
    ang_nodes,
    render_qual,
    qn_n,
    qn_l,
    qn_m,
    hybridized_dropdown,
    rad_wavefunction,
    rad_prob_density,
    rad_prob_distr,
    cross_section,
    isosurface_3d,
);

pub(crate) const DESC: Descriptions = Descriptions {
    qn_dropdown: "Select the orbital to display. Use the \"Real (Full)\" orbital type to select arbitrary quantum numbers.",
    supplement: "Additional visualizations that highlight different aspects of orbitals.",
    rad_nodes: "Concentric, spherical surfaces where there is exactly 0 possibility of finding an electron. There are n−ℓ−1 radial nodes.",
    ang_nodes: "Planar or conical surfaces where there is exactly 0 possibility of finding an electron. There are ℓ angular nodes.",
    render_qual: "More points can be sampled for higher quality visualizations, at the cost of performance.",
    qn_n: "n corresponds to the overall energy level of the orbital.",
    qn_l: "ℓ describes the amount of angular momentum possessed by the electron.",
    qn_m: "m corresponds to how angular momentum is aligned. Orbitals with the same n and ℓ values are \"degenerate\" – they all have the same energy.",
    hybridized_dropdown: "Select the hybridized orbital to display. There are multiple orbitals with the same kind, each with a different orientation.",
    rad_wavefunction: "This is the radial component of the wavefunction. Radial nodes are found where the curve crosses 0.",
    rad_prob_density: "The radial probability density gives the probability of finding the electron at a point of radius r from the nucleus.",
    rad_prob_distr: "The radial probability distribution gives the total probability of finding the electron on a shell of radius r.",
    cross_section: "This plot displays the behavior of the wavefunction along a given plane (drawn in orange). The wavefunction value is displayed using the third vertical axis, and a contour plot is also projected onto the plane.",
    isosurface_3d: "This plot displays all the surfaces where the wavefunction is exactly equal to a certain value. It provides a general sense of the structure of the orbital, but does not indicate its probabilistic nature.",
};
