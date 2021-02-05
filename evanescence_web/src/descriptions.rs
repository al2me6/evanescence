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
    qn_m
);

pub(crate) const DESC: Descriptions = Descriptions {
    qn_dropdown:
        "Select the orbital to display. Choose \"Custom\" to select arbitrary quantum numbers.",
    supplement: "Additional visualizations that highlight different aspects of orbitals.",
    rad_nodes: "Concentric, spherical surfaces where there is exactly 0 possibility of finding an electron. There are n−ℓ−1 radial nodes.",
    ang_nodes: "Planar or conical surfaces where there is exactly 0 possibility of finding an electron. There are ℓ angular nodes.",
    render_qual: "Sample more points for higher quality visualizations, at the cost of performance.",
    qn_n: "n corresponds to the overall energy level of the orbital.",
    qn_l: "ℓ describes the amount of angular momentum possessed by the electron.",
    qn_m: "m corresponds to how angular momentum is aligned. Orbitals with the same n and ℓ values are \"degenerate\" – they all have the same energy.",
};
