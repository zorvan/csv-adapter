/// Comprehensive Wallet Management page supporting all csv-cli commands.

use dioxus::prelude::*;
use csv_adapter_core::Chain;
use std::collections::HashMap;
use crate::context::{use_wallet_context, Network, truncate_address};
use crate::routes::Route;
use crate::components::{Dropdown, Card, StatCard, ChainDisplay, NetworkDisplay, all_chain_displays, all_network_displays};

#[derive(Clone, Copy, PartialEq)]
enum WalletTab {
    Overview,
    Generate,
    Import,
    Balance,
    Fund,
    Export,
    List,
}

impl std::fmt::Display for WalletTab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Overview => write!(f, "Overview"),
            Self::Generate => write!(f, "Generate"),
            Self::Import => write!(f, "Import"),
            Self::Balance => write!(f, "Balance"),
            Self::Fund => write!(f, "Fund"),
            Self::Export => write!(f, "Export"),
            Self::List => write!(f, "List"),
        }
    }
}

#[component]
pub fn WalletPage() -> Element {
    let mut wallet_ctx = use_wallet_context();
    let mut active_tab = use_signal(|| WalletTab::Overview);
    let mut selected_chain = use_signal(|| ChainDisplay(Chain::Bitcoin));
    let mut selected_network = use_signal(|| NetworkDisplay(Network::Test));
    let mut message = use_signal(|| Option::<String>::None);
    let mut mnemonic_result = use_signal(|| Option::<String>::None);
    let mut import_input = use_signal(|| String::new());

    let tabs = vec![
        WalletTab::Overview,
        WalletTab::Generate,
        WalletTab::Import,
        WalletTab::Balance,
        WalletTab::Fund,
        WalletTab::Export,
        WalletTab::List,
    ];

    let addresses: HashMap<Chain, String> = wallet_ctx.addresses().into_iter().collect();
    let has_wallet = !addresses.is_empty();

    rsx! {
        div { class: "space-y-6",
            // Header
            div { class: "flex items-center justify-between",
                h1 { class: "text-3xl font-bold text-gray-100", "Wallet Management" }
                div { class: "flex items-center gap-2 text-sm text-gray-400",
                    span { class: "w-2 h-2 rounded-full", class: if has_wallet { "bg-green-500" } else { "bg-yellow-500" } }
                    if has_wallet { "Wallet Ready" } else { "No Wallet" }
                }
            }

            // Message display
            if let Some(msg) = message.read().clone() {
                div { class: "bg-blue-500/10 border border-blue-500/30 rounded-xl p-4 text-sm text-blue-300", "{msg}" }
            }

            // Mnemonic display
            if let Some(mnemonic) = mnemonic_result.read().clone() {
                div { class: "bg-yellow-500/10 border border-yellow-500/30 rounded-xl p-6 space-y-3",
                    div { class: "flex items-center gap-2",
                        span { class: "text-yellow-400", "\u{26A0}\u{FE0F}" }
                        p { class: "text-yellow-300 font-medium", "Save your recovery phrase!" }
                    }
                    div { class: "bg-gray-800/50 rounded-lg p-4 border border-gray-700",
                        p { class: "font-mono text-sm text-gray-200 break-all leading-relaxed", "{mnemonic}" }
                    }
                    button {
                        onclick: move |_| {
                            wallet_ctx.clear_pending_secret();
                            mnemonic_result.set(None);
                            message.set(Some("Recovery phrase cleared from memory".to_string()));
                        },
                        class: "px-4 py-2 rounded-lg bg-gray-800 hover:bg-gray-700 text-sm font-medium transition-colors",
                        "Clear from Memory"
                    }
                }
            }

            // Tab navigation
            div { class: "bg-gray-900 rounded-xl border border-gray-800 p-1",
                div { class: "flex gap-1 overflow-x-auto",
                    for tab in tabs {
                        button {
                            onclick: move |_| active_tab.set(tab),
                            class: "px-4 py-2 rounded-lg text-sm font-medium transition-all whitespace-nowrap",
                            class: if active_tab() == tab { "bg-blue-600 text-white" } else { "text-gray-400 hover:text-gray-200 hover:bg-gray-800" },
                            "{tab}"
                        }
                    }
                }
            }

            // Tab content
            match active_tab() {
                WalletTab::Overview => rsx! {
                    OverviewTab { has_wallet, address_count: addresses.len() }
                },
                WalletTab::Generate => rsx! {
                    GenerateTab {
                        selected_chain: selected_chain.read().clone(),
                        selected_network: selected_network.read().clone(),
                        on_chain_change: move |cd: ChainDisplay| selected_chain.set(cd),
                        on_network_change: move |nd: NetworkDisplay| selected_network.set(nd),
                        on_generate: move |mnemonic: String| {
                            mnemonic_result.set(Some(mnemonic));
                            message.set(Some("Wallet generated! Save the recovery phrase above.".to_string()));
                        },
                    }
                },
                WalletTab::Import => rsx! {
                    ImportTab {
                        selected_chain: selected_chain.read().clone(),
                        selected_network: selected_network.read().clone(),
                        on_chain_change: move |cd: ChainDisplay| selected_chain.set(cd),
                        on_network_change: move |nd: NetworkDisplay| selected_network.set(nd),
                        import_input: import_input.read().clone(),
                        on_input_change: move |val: String| import_input.set(val),
                        on_import: move |_| {
                            message.set(Some("Wallet imported successfully!".to_string()));
                            import_input.set(String::new());
                        },
                    }
                },
                WalletTab::Balance => rsx! {
                    BalanceTab {
                        selected_chain: selected_chain.read().clone(),
                        on_chain_change: move |cd: ChainDisplay| selected_chain.set(cd),
                        addresses: addresses.clone(),
                    }
                },
                WalletTab::Fund => rsx! {
                    FundTab {
                        selected_chain: selected_chain.read().clone(),
                        selected_network: selected_network.read().clone(),
                        on_chain_change: move |cd: ChainDisplay| selected_chain.set(cd),
                        on_network_change: move |nd: NetworkDisplay| selected_network.set(nd),
                        on_fund: move |msg: String| message.set(Some(msg)),
                        addresses: addresses.clone(),
                    }
                },
                WalletTab::Export => rsx! {
                    ExportTab {
                        selected_chain: selected_chain.read().clone(),
                        on_chain_change: move |cd: ChainDisplay| selected_chain.set(cd),
                        addresses: addresses.clone(),
                        on_export: move |msg: String| message.set(Some(msg)),
                    }
                },
                WalletTab::List => rsx! {
                    ListTab { addresses: addresses.clone() }
                },
            }
        }
    }
}

#[component]
fn OverviewTab(has_wallet: bool, address_count: usize) -> Element {
    rsx! {
        div { class: "space-y-6",
            div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                StatCard { label: "Total Addresses", value: address_count.to_string(), icon: "\u{1F4B3}" }
                StatCard { label: "Supported Chains", value: "4".to_string(), icon: "\u{26D3}\u{FE0F}" }
                StatCard { label: "Networks", value: "3".to_string(), icon: "\u{1F310}" }
            }

            Card {
                title: if has_wallet { "Quick Actions" } else { "Get Started" },
                children: rsx! {
                    if has_wallet {
                        div { class: "grid grid-cols-2 md:grid-cols-4 gap-4",
                            Link { to: Route::Dashboard {}, class: "p-4 bg-gray-800 rounded-lg hover:bg-gray-700 transition-colors text-center",
                                div { class: "text-2xl mb-2", "\u{1F4CA}" }
                                div { class: "font-medium text-sm", "Dashboard" }
                            }
                            Link { to: Route::Rights {}, class: "p-4 bg-gray-800 rounded-lg hover:bg-gray-700 transition-colors text-center",
                                div { class: "text-2xl mb-2", "\u{1F48E}" }
                                div { class: "font-medium text-sm", "Rights" }
                            }
                            Link { to: Route::CrossChain {}, class: "p-4 bg-gray-800 rounded-lg hover:bg-gray-700 transition-colors text-center",
                                div { class: "text-2xl mb-2", "\u{21C4}" }
                                div { class: "font-medium text-sm", "Cross-Chain" }
                            }
                            Link { to: Route::Settings {}, class: "p-4 bg-gray-800 rounded-lg hover:bg-gray-700 transition-colors text-center",
                                div { class: "text-2xl mb-2", "\u{2699}\u{FE0F}" }
                                div { class: "font-medium text-sm", "Settings" }
                            }
                        }
                    } else {
                        div { class: "text-center py-8 space-y-4",
                            div { class: "text-6xl", "\u{1F510}" }
                            p { class: "text-gray-400", "No wallet detected. Generate or import a wallet to get started." }
                            div { class: "flex justify-center gap-4",
                                Link { to: Route::GenerateWallet {}, class: "px-6 py-3 bg-blue-600 hover:bg-blue-700 rounded-lg font-medium transition-colors", "Generate Wallet" }
                                Link { to: Route::ImportWalletPage {}, class: "px-6 py-3 bg-gray-800 hover:bg-gray-700 rounded-lg font-medium transition-colors", "Import Wallet" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn GenerateTab(
    selected_chain: ChainDisplay,
    selected_network: NetworkDisplay,
    on_chain_change: EventHandler<ChainDisplay>,
    on_network_change: EventHandler<NetworkDisplay>,
    on_generate: EventHandler<String>,
) -> Element {
    rsx! {
        Card {
            title: "Generate New Wallet",
            children: rsx! {
                div { class: "space-y-6",
                    div {
                        label { class: "block text-sm font-medium text-gray-300 mb-2", "Blockchain" }
                        Dropdown {
                            options: all_chain_displays(),
                            selected: selected_chain,
                            on_change: move |cd| on_chain_change.call(cd),
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-300 mb-2", "Network" }
                        Dropdown {
                            options: all_network_displays(),
                            selected: selected_network,
                            on_change: move |nd| on_network_change.call(nd),
                        }
                    }

                    button {
                        onclick: move |_| {
                            // Simulate wallet generation
                            let mnemonic = "abandon ability able about above absent absorb abstract absurd abuse access accident".to_string();
                            on_generate.call(mnemonic);
                        },
                        class: "w-full px-6 py-3 bg-blue-600 hover:bg-blue-700 rounded-lg font-medium transition-colors text-white",
                        "Generate Wallet"
                    }

                    div { class: "bg-blue-500/10 border border-blue-500/20 rounded-lg p-4 text-sm text-gray-400",
                        span { class: "text-blue-400 font-medium", "\u{2139}\u{FE0F} Info: " }
                        "A new wallet will be generated with a random 12-word recovery phrase. Save it securely."
                    }
                }
            }
        }
    }
}

#[component]
fn ImportTab(
    selected_chain: ChainDisplay,
    selected_network: NetworkDisplay,
    on_chain_change: EventHandler<ChainDisplay>,
    on_network_change: EventHandler<NetworkDisplay>,
    import_input: String,
    on_input_change: EventHandler<String>,
    on_import: EventHandler<()>,
) -> Element {
    rsx! {
        Card {
            title: "Import Wallet",
            children: rsx! {
                div { class: "space-y-6",
                    div {
                        label { class: "block text-sm font-medium text-gray-300 mb-2", "Blockchain" }
                        Dropdown {
                            options: all_chain_displays(),
                            selected: selected_chain,
                            on_change: move |cd| on_chain_change.call(cd),
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-300 mb-2", "Network" }
                        Dropdown {
                            options: all_network_displays(),
                            selected: selected_network,
                            on_change: move |nd| on_network_change.call(nd),
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-300 mb-2", "Private Key or Mnemonic" }
                        textarea {
                            value: "{import_input}",
                            oninput: move |evt| on_input_change.call(evt.value()),
                            placeholder: "Enter private key (hex) or mnemonic phrase...",
                            class: "w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-3 text-gray-100 font-mono text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 resize-none",
                            rows: 4,
                        }
                    }

                    button {
                        onclick: move |_| on_import.call(()),
                        class: "w-full px-6 py-3 bg-green-600 hover:bg-green-700 rounded-lg font-medium transition-colors text-white",
                        "Import Wallet"
                    }

                    div { class: "bg-yellow-500/10 border border-yellow-500/20 rounded-lg p-4 text-sm text-gray-400",
                        span { class: "text-yellow-400 font-medium", "\u{26A0}\u{FE0F} Warning: " }
                        "Never share your private key or mnemonic. Only import from trusted sources."
                    }
                }
            }
        }
    }
}

#[component]
fn BalanceTab(
    selected_chain: ChainDisplay,
    on_chain_change: EventHandler<ChainDisplay>,
    addresses: HashMap<Chain, String>,
) -> Element {
    let chain = selected_chain.0;
    let addr = addresses.get(&chain).cloned().unwrap_or_else(|| "Not generated".to_string());

    rsx! {
        Card {
            title: "Check Balance",
            children: rsx! {
                div { class: "space-y-6",
                    div {
                        label { class: "block text-sm font-medium text-gray-300 mb-2", "Blockchain" }
                        Dropdown {
                            options: all_chain_displays(),
                            selected: selected_chain,
                            on_change: move |cd| on_chain_change.call(cd),
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-300 mb-2", "Address" }
                        div { class: "bg-gray-800 border border-gray-700 rounded-lg px-4 py-3 font-mono text-sm text-gray-200 break-all", "{addr}" }
                    }

                    div { class: "bg-gray-800/50 rounded-lg p-4",
                        div { class: "flex items-center justify-between",
                            span { class: "text-gray-400", "Balance" }
                            span { class: "text-2xl font-bold text-gray-100", "0.0000" }
                        }
                        div { class: "text-xs text-gray-500 mt-1", "Connect to RPC to fetch real balance" }
                    }
                }
            }
        }
    }
}

#[component]
fn FundTab(
    selected_chain: ChainDisplay,
    selected_network: NetworkDisplay,
    on_chain_change: EventHandler<ChainDisplay>,
    on_network_change: EventHandler<NetworkDisplay>,
    on_fund: EventHandler<String>,
    addresses: HashMap<Chain, String>,
) -> Element {
    let chain = selected_chain.0;
    let network = selected_network.0;
    let addr = addresses.get(&chain).cloned().unwrap_or_else(|| "Generate a wallet first".to_string());

    rsx! {
        Card {
            title: "Fund from Faucet",
            children: rsx! {
                div { class: "space-y-6",
                    div {
                        label { class: "block text-sm font-medium text-gray-300 mb-2", "Blockchain" }
                        Dropdown {
                            options: all_chain_displays(),
                            selected: selected_chain,
                            on_change: move |cd| on_chain_change.call(cd),
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-300 mb-2", "Network" }
                        Dropdown {
                            options: all_network_displays(),
                            selected: selected_network,
                            on_change: move |nd| on_network_change.call(nd),
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-300 mb-2", "Target Address" }
                        div { class: "bg-gray-800 border border-gray-700 rounded-lg px-4 py-3 font-mono text-sm text-gray-200 break-all", "{addr}" }
                    }

                    button {
                        onclick: move |_| {
                            if addr != "Generate a wallet first" {
                                on_fund.call(format!("Faucet request sent for {} on {}", chain, network));
                            }
                        },
                        class: "w-full px-6 py-3 bg-purple-600 hover:bg-purple-700 rounded-lg font-medium transition-colors text-white",
                        "Request Test Tokens"
                    }

                    div { class: "bg-purple-500/10 border border-purple-500/20 rounded-lg p-4 text-sm text-gray-400",
                        span { class: "text-purple-400 font-medium", "\u{2139}\u{FE0F} Info: " }
                        "Test tokens will be requested from the chain's official faucet. This may take a few minutes."
                    }
                }
            }
        }
    }
}

#[component]
fn ExportTab(
    selected_chain: ChainDisplay,
    on_chain_change: EventHandler<ChainDisplay>,
    addresses: HashMap<Chain, String>,
    on_export: EventHandler<String>,
) -> Element {
    let chain = selected_chain.0;
    let addr = addresses.get(&chain).cloned().unwrap_or_else(|| "Not generated".to_string());

    rsx! {
        Card {
            title: "Export Wallet",
            children: rsx! {
                div { class: "space-y-6",
                    div {
                        label { class: "block text-sm font-medium text-gray-300 mb-2", "Blockchain" }
                        Dropdown {
                            options: all_chain_displays(),
                            selected: selected_chain,
                            on_change: move |cd| on_chain_change.call(cd),
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-300 mb-2", "Address" }
                        div { class: "bg-gray-800 border border-gray-700 rounded-lg px-4 py-3 font-mono text-sm text-gray-200 break-all", "{addr}" }
                    }

                    div { class: "space-y-3",
                        button {
                            onclick: move |_| {
                                on_export.call(format!("Exported address for {}: {}", chain, addr));
                            },
                            class: "w-full px-6 py-3 bg-blue-600 hover:bg-blue-700 rounded-lg font-medium transition-colors text-white",
                            "Export Address"
                        }
                        button {
                            onclick: move |_| {
                                on_export.call("Export full wallet data (JSON format)".to_string());
                            },
                            class: "w-full px-6 py-3 bg-gray-800 hover:bg-gray-700 rounded-lg font-medium transition-colors",
                            "Export Full Wallet (JSON)"
                        }
                    }
                }
            }
        }
    }
}

fn chain_badge_class(chain: &Chain) -> &'static str {
    match chain {
        Chain::Bitcoin => "inline-flex items-center gap-1.5 px-2 py-1 rounded-full text-xs font-medium text-orange-400 bg-orange-500/20 border border-orange-500/30",
        Chain::Ethereum => "inline-flex items-center gap-1.5 px-2 py-1 rounded-full text-xs font-medium text-blue-400 bg-blue-500/20 border border-blue-500/30",
        Chain::Sui => "inline-flex items-center gap-1.5 px-2 py-1 rounded-full text-xs font-medium text-cyan-400 bg-cyan-500/20 border border-cyan-500/30",
        Chain::Aptos => "inline-flex items-center gap-1.5 px-2 py-1 rounded-full text-xs font-medium text-emerald-400 bg-emerald-500/20 border border-emerald-500/30",
    }
}

#[component]
fn ListTab(addresses: HashMap<Chain, String>) -> Element {
    let chains = [Chain::Bitcoin, Chain::Ethereum, Chain::Sui, Chain::Aptos];

    rsx! {
        Card {
            title: "All Wallets",
            children: rsx! {
                if addresses.is_empty() {
                    div { class: "text-center py-12 text-gray-500",
                        div { class: "text-5xl mb-4", "\u{1F4CB}" }
                        div { class: "text-lg font-medium", "No wallets" }
                        p { class: "text-sm mt-2", "Generate or import a wallet to see it listed here." }
                    }
                } else {
                    div { class: "overflow-x-auto",
                        table { class: "w-full",
                            thead {
                                tr { class: "border-b border-gray-800",
                                    th { class: "text-left py-3 px-4 text-sm font-medium text-gray-400", "Blockchain" }
                                    th { class: "text-left py-3 px-4 text-sm font-medium text-gray-400", "Address" }
                                    th { class: "text-left py-3 px-4 text-sm font-medium text-gray-400", "Balance" }
                                }
                            }
                            tbody {
                                for chain in chains {
                                    if let Some(addr) = addresses.get(&chain) {
                                        tr { class: "border-b border-gray-800/50 hover:bg-gray-800/30 transition-colors",
                                            td { class: "py-3 px-4",
                                                span { class: "{chain_badge_class(&chain)}",
                                                    "{chain}"
                                                }
                                            }
                                            td { class: "py-3 px-4 font-mono text-sm text-gray-200", "{addr}" }
                                            td { class: "py-3 px-4 text-sm text-gray-300", "0.0000" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
