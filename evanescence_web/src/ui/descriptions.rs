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
    nodes_rad,
    nodes_ang,
    render_qual,
    qn_n,
    qn_l,
    qn_m,
    hybrid_dropdown,
    show_symmetry,
    rad_wavefunction,
    rad_prob_distr,
    cross_section_wavefunction,
    cross_section_prob_density,
    isosurface_3d,
    nodes_hybrid,
    interatomic_separation,
);

pub(crate) const DESC: Descriptions = Descriptions {
    qn_dropdown: "Select the orbital to display. Use the \"Real (Full)\" orbital type to select arbitrary quantum numbers.",
    supplement: "Select additional visualizations to highlight specific features of orbitals.",
    nodes_rad: "Draw radial nodes in green, if they exist. These are concentric, spherical surfaces where there is exactly 0 possibility of finding an electron. There are <i>n</i>−<i>ℓ</i>−1 radial nodes.",
    nodes_ang: "Draw angular nodes in purple, if they exist. These are planar or conical surfaces where there is exactly 0 possibility of finding an electron. There are <i>ℓ</i> angular nodes.",
    render_qual: "More points can be sampled for higher quality visualizations, at the cost of performance.",
    qn_n: "<i>n</i> corresponds to the overall energy level of the orbital.",
    qn_l: "<i>ℓ</i> describes the amount of angular momentum possessed by the electron.",
    qn_m: "<i>m</i> corresponds to how angular momentum is aligned.",
    hybrid_dropdown: "Select the hybridization to display. There are multiple orbitals of each kind, with different orientations.",
    show_symmetry: "Draw silhouettes for all orbitals of this type to display symmetry.",
    rad_wavefunction: "This is the radial component of the wavefunction. Radial nodes are found where the curve crosses 0.",
    rad_prob_distr: "The probability of finding the electron within a shell of inner radius <i>a</i> and outer radius <i>b</i> is given by the area under the portion of this curve running from <i>a</i> to <i>b</i>. By definition, the total area under this curve from 0 to infinity, corresponding to the probability of finding the electron anywhere in space, is 1.",
    cross_section_wavefunction: "This plot displays the behavior of the wavefunction along a given plane (drawn in orange). The wavefunction value is displayed using the third vertical axis and a contour plot is projected onto the plane.",
    cross_section_prob_density: "This plot displays the orbital's probability density along a given plane (drawn in orange). The value is displayed using the third vertical axis and a contour plot is projected onto the plane.",
    isosurface_3d: "This plot displays all the surfaces where the probability density is equal to a certain value. It provides a general sense of the structure of the orbital, but does not indicate its probabilistic nature.",
    nodes_hybrid: "Draw nodes, or surfaces where there is exactly 0 possibility of finding an electron, in purple.",
    interatomic_separation: "The distance between the two protons, in Bohr radii.",
};
