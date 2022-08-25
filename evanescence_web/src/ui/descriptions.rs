macro_rules! descriptions {
    ($($name:ident : $value:literal),+ $(,)?) => {
        pub struct Descriptions {
            $(pub $name: &'static str),+
        }

        pub const DESC: Descriptions = Descriptions {
            $($name: $value),+
        };
    }
}

descriptions! {
    qn_dropdown: "Select the orbital to display. Use the \"Real (Full)\" orbital type to select arbitrary quantum numbers.",
    supplement: "Select additional visualizations to highlight specific features of orbitals.",
    nodes_rad: "Draw radial nodes in green, if they exist. These are concentric, spherical surfaces where there is exactly 0 possibility of finding an electron. There are <i>n</i>−<i>ℓ</i>−1 radial nodes.",
    nodes_ang: "Draw angular nodes in purple, if they exist. These are planar or conical surfaces where there is exactly 0 possibility of finding an electron. There are <i>ℓ</i> angular nodes.",
    render_qual: "More points can be sampled for higher quality visualizations, at the cost of performance.",
    qn_n: "<i>n</i> corresponds to the overall energy level of the orbital.",
    qn_l: "<i>ℓ</i> describes the amount of angular momentum possessed by the electron.",
    qn_m: "<i>m</i> corresponds to how angular momentum is aligned.",
    instant_apply: "When changing multiple quantum number dropdowns, applying the changes manually can reduce the number of pauses experienced.",
    hybrid_dropdown: "Select the hybridization to display. There are multiple orbitals of each kind, with different orientations.",
    show_symmetry: "Draw silhouettes for all orbitals of this type to display symmetry.",
    rad_wavefunction: "This is the radial component of the wavefunction. Radial nodes are found where the curve crosses 0.",
    rad_prob_distr: "The probability of finding the electron within a shell of inner radius <i>a</i> and outer radius <i>b</i> is given by the area under the portion of this curve running from <i>a</i> to <i>b</i>. By definition, the total area under this curve from 0 to infinity, corresponding to the probability of finding the electron anywhere in space, is 1.",
    rad_cumulative: "The probability of finding the electron within a given distance from the nucleus, given by integrating the radial probability distribution from 0 to <i>r</i>.",
    cross_section_wavefunction: "This plot displays the behavior of the wavefunction along a given plane (drawn in orange). The wavefunction value is displayed using the third vertical axis and a contour plot is projected onto the plane.",
    cross_section_prob_density: "This plot displays the orbital's probability density along a given plane (drawn in orange). The value is displayed using the third vertical axis and a contour plot is projected onto the plane.",
    isosurface_3d: "This plot displays all the surfaces where the probability density is equal to a certain value. It provides a general sense of the structure of the orbital, but does not indicate its probabilistic nature.",
    nodes_hybrid: "Draw nodes, or surfaces where there is exactly 0 possibility of finding an electron, in purple.",
    interatomic_separation: "The distance between the two protons, in Bohr radii.",
}
