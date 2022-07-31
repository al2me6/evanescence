use evanescence_core::numerics::spherical_harmonics::RealSphericalHarmonic;
use evanescence_core::orbital::quantum_numbers::Lm;
use evanescence_core::orbital::{self, Qn};
use evanescence_web::components::{Button, CheckBox, Dropdown, Tooltip};
use evanescence_web::plotters::Quality;
use evanescence_web::presets::{HybridPreset, QnPreset};
use evanescence_web::state::{Mode, State, StateDispatch, Visualization};
use evanescence_web::utils;
use itertools::Itertools;
use strum::IntoEnumIterator;
use yew::prelude::*;
use yewdux::prelude::*;

use super::descriptions::DESC;

fn td_tooltip(text: &'static str, tooltip: &'static str) -> Html {
    html! {
        <td><Tooltip { text } { tooltip } /></td>
    }
}

pub struct Controls {}

impl Component for Controls {
    type Message = ();
    type Properties = StateDispatch;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let dispatch = ctx.props();
        let state = dispatch.state();

        let selectors = match state.mode() {
            Mode::RealSimple | Mode::RealFull => self.real_modes_controls(dispatch),
            Mode::Complex => self.qn_pickers(dispatch),
            Mode::Hybrid => self.hybrid_picker(dispatch),
            // Mode::Mo => self.mo_picker(dispatch),
        };

        html! {
            <div id = "controls">
                <table>
                    { selectors }
                    <tr>
                        { td_tooltip("Show supplemental visualization:", DESC.supplement) }
                        <td><Dropdown<Visualization>
                            id = "supplement-picker"
                            on_change = { dispatch.reduce_callback_with(State::set_supplement) }
                            options = { state.available_supplements() }
                            selected = { state.supplement() }
                        /></td>
                    </tr>
                    <tr>
                        { td_tooltip("Render quality:", DESC.render_qual) }
                        <td><Dropdown<Quality>
                            id = "quality-picker"
                            on_change = { dispatch.reduce_callback_with(State::set_quality) }
                            options = { Quality::iter().collect_vec() }
                            custom_display = { Quality::iter().map(Quality::to_text).collect_vec() }
                            selected = { state.quality() }
                        /></td>
                    </tr>
                </table>
            </div>
        }
    }
}

impl Controls {
    fn real_modes_controls(&self, dispatch: &StateDispatch) -> Html {
        let state = dispatch.state();
        assert!(state.mode().is_real_or_simple());

        html! {
            <>
            if state.mode().is_real() {
                { self.qn_pickers(dispatch) }
            } else {
                <tr>
                    { td_tooltip("Select orbital:", DESC.qn_dropdown) }
                    <td><Dropdown<QnPreset>
                        id = "preset_picker"
                        on_change = { dispatch.reduce_callback_with(State::set_qn_preset) }
                        options = { QnPreset::presets() }
                        selected = { state.qn_preset() }
                    /></td>
                </tr>
            }
            <tr>
                <td/>
                <td><CheckBox
                    id = "radial-nodes-toggle"
                    on_change = { dispatch.reduce_callback_with(State::set_nodes_rad) }
                    checked = { state.nodes_rad() }
                    label = "Show radial nodes"
                    tooltip = { DESC.nodes_rad }
                /></td>
            </tr>
            <tr>
                <td/>
                <td><CheckBox
                    id = "angular-nodes-toggle"
                    on_change = { dispatch.reduce_callback_with(State::set_nodes_ang) }
                    checked = { state.nodes_ang() }
                    label = "Show angular nodes"
                    tooltip = { DESC.nodes_ang }
                /></td>
            </tr>
            </>
        }
    }

    fn qn_pickers(&self, dispatch: &StateDispatch) -> Html {
        let state = dispatch.state();
        assert!(state.mode().is_real() || state.mode().is_complex());
        html! {
            <QnPickers
                qn = { *state.qn() }
                mode = { state.mode() }
                instant = { state.instant_apply() }
                on_apply = { dispatch.reduce_callback_with(State::set_qn) }
                on_toggle_instant = { dispatch.reduce_callback_with(State::set_instant_apply) }
            />
        }
    }

    fn hybrid_picker(&self, dispatch: &StateDispatch) -> Html {
        let state = dispatch.state();
        assert!(state.mode().is_hybrid());

        html! {
            <>
            <tr>
                { td_tooltip("Select hybridization:", DESC.hybrid_dropdown) }
                <td><Dropdown<HybridPreset>
                    id = "preset_picker"
                    on_change = { dispatch.reduce_callback_with(State::set_hybrid_preset) }
                    options = { HybridPreset::presets() }
                    selected = { state.hybrid_preset() }
                /></td>
            </tr>
            <tr>
                <td/>
                <td><CheckBox
                    id = "show-symmetry-toggle"
                    on_change = { dispatch.reduce_callback_with(State::set_silhouettes) }
                    checked ={ state.silhouettes() }
                    label = "Show symmetry"
                    tooltip = { DESC.show_symmetry }
                /></td>
            </tr>
            <tr>
                <td/>
                <td><CheckBox
                    id = "hybrid-nodes-toggle"
                    on_change = { dispatch.reduce_callback_with(State::set_nodes) }
                    checked = { state.nodes() }
                    label = "Show nodes"
                    tooltip = { DESC.nodes_hybrid }
                /></td>
            </tr>
            </>
        }
    }

    //     fn mo_picker(&self, dispatch: &AppDispatch) -> Html {
    //         let state = dispatch.state();
    //         assert!(state.mode().is_mo());
    //         html! {
    //             <>
    //             <tr>
    //                 { td_tooltip("Select molecular orbital:", DESC.hybrid_dropdown) }
    //                 <td><Dropdown<MoPreset>
    //                     id = "preset_picker"
    //                     on_change = { dispatch.reduce_callback_with(State::set_mo_preset) }
    //                     options = { MoPreset::presets() }
    //                     selected = { state.mo_preset() }
    //                 /></td>
    //             </tr>
    //             <tr>
    //                 { td_tooltip("Interatomic separation:", DESC.interatomic_separation) }
    //                 <td><Slider
    //                     id = "sep-slider"
    //                     on_change = { dispatch.reduce_callback_with(State::set_separation) }
    //                     min = 0.0
    //                     value = { state.separation() }
    //                     max = 10.0
    //                     step = 0.1
    //                     value_postfix = " A₀"
    //                 /></td>
    //             </tr>
    //             <tr>
    //                 <td/>
    //                 <td><CheckBox
    //                     id = "mo-nodes-toggle"
    //                     on_change = { dispatch.reduce_callback_with(State::set_nodes) }
    //                     checked = { state.nodes() }
    //                     label = "Show nodes"
    //                     tooltip = { DESC.nodes_hybrid }
    //                 /></td>
    //             </tr>
    //             </>
    //         }
    //     }
}

pub struct QnPickers {
    qn: Qn,
}

#[derive(Clone, PartialEq, Properties)]
pub struct QnPickersProps {
    qn: Qn,
    mode: Mode,
    instant: bool,
    on_apply: Callback<Qn>,
    on_toggle_instant: Callback<bool>,
}

#[derive(PartialEq, Eq)]
pub enum QnPickersMsg {
    N(u32),
    L(u32),
    M(i32),
    SetInstant(bool),
    Apply,
}

impl Component for QnPickers {
    type Message = QnPickersMsg;
    type Properties = QnPickersProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self { qn: ctx.props().qn }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        use QnPickersMsg as Msg;
        match msg {
            Msg::N(n) => self.qn.set_n_clamping(n).unwrap(),
            Msg::L(l) => self.qn.set_l_clamping(l).unwrap(),
            Msg::M(m) => self.qn.set_m(m).unwrap(),
            Msg::SetInstant(instant) => ctx.props().on_toggle_instant.emit(instant),
            Msg::Apply => ctx.props().on_apply.emit(self.qn),
        }
        if ctx.props().instant && msg != Msg::Apply || msg == Msg::SetInstant(true) {
            ctx.link().send_message(Msg::Apply);
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let qn = &self.qn;

        let l_options = Qn::enumerate_l_for_n(qn.n()).unwrap().collect_vec();
        let m_options = Qn::enumerate_m_for_l(qn.l()).collect_vec();

        let format_l = |l: &u32| match orbital::atomic::subshell_name(*l) {
            Some(subshell) => format!("{l} [{subshell}]"),
            None => l.to_string(),
        };

        let format_m = |m: &i32| match ctx.props().mode {
            Mode::RealFull => match RealSphericalHarmonic::expression(Lm::new(qn.l(), *m).unwrap())
            {
                Some(expression) if !expression.is_empty() => {
                    format!("{} [ {expression} ]", utils::fmt_replace_minus(m))
                }
                _ => utils::fmt_replace_minus(m),
            },
            Mode::Complex => utils::fmt_replace_minus(m),
            Mode::RealSimple | Mode::Hybrid => unreachable!(),
        };

        html! {
            <>
            <tr>
                { td_tooltip("Principal quantum number <i>n</i>:", DESC.qn_n) }
                <td><Dropdown<u32>
                    id = "n-picker"
                    on_change = { ctx.link().callback(QnPickersMsg::N) }
                    options = { (1..=evanescence_web::MAX_N).collect_vec() }
                    selected = { qn.n() }
                /></td>
            </tr>
            <tr>
                { td_tooltip("Azimuthal quantum number <i>ℓ</i>:", DESC.qn_l) }
                <td><Dropdown<u32>
                    id = "l-picker"
                    on_change = { ctx.link().callback(QnPickersMsg::L) }
                    options = { l_options }
                    custom_display = { l_options.iter().map(format_l).collect_vec() }
                    selected = { qn.l() }
                /></td>
            </tr>
            <tr>
                { td_tooltip("Magnetic quantum number <i>m</i>:", DESC.qn_m) }
                <td><Dropdown<i32>
                    id = "m-picker"
                    on_change = { ctx.link().callback(QnPickersMsg::M) }
                    options = { m_options }
                    custom_display = { m_options.iter().map(format_m).collect_vec() }
                    selected = { qn.m() }
                /></td>
            </tr>
            <tr>
                <td>
                    <Button
                        id = "qn-apply-button"
                        enabled = { ctx.props().qn != self.qn }
                        on_click = { ctx.link().callback(|_| QnPickersMsg::Apply) }
                        text = { if ctx.props().qn == self.qn { "QNs applied" } else { "Apply QNs" } }
                        hover = "Apply selected quantum numbers"
                    />
                </td>
                <td class = "qn-apply-selector">
                    <CheckBox
                        id = "instant-apply"
                        on_change = { ctx.link().callback(QnPickersMsg::SetInstant) }
                        checked = { ctx.props().instant }
                        label = "Apply instantly"
                        tooltip = { DESC.instant_apply }
                    />
                </td>
            </tr>
            </>
        }
    }
}
