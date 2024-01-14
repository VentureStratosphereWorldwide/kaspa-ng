use crate::app::{GIT_DESCRIBE, VERSION};
use crate::imports::*;
use crate::settings::NodeMemoryScale;
use crate::utils::Arglist;
use kaspa_core::kaspad_env;
#[cfg(not(target_arch = "wasm32"))]
pub use kaspad_lib::args::Args;

fn user_agent_comment() -> String {
    format!(
        "/{}:{}/kaspa-ng:{}-{}/",
        kaspad_env::name(),
        kaspad_env::version(),
        VERSION,
        GIT_DESCRIBE
    )
}

#[derive(Debug, Clone)]
pub struct Config {
    network: Network,
    enable_upnp: bool,
    enable_grpc: bool,
    grpc_network_interface: NetworkInterfaceConfig,
    kaspad_daemon_args_enable: bool,
    kaspad_daemon_args: String,
    memory_scale: NodeMemoryScale,
}

impl From<NodeSettings> for Config {
    fn from(node_settings: NodeSettings) -> Self {
        Self {
            network: node_settings.network,
            enable_upnp: node_settings.enable_upnp,
            enable_grpc: node_settings.enable_grpc,
            grpc_network_interface: node_settings.grpc_network_interface,
            kaspad_daemon_args_enable: node_settings.kaspad_daemon_args_enable,
            kaspad_daemon_args: node_settings.kaspad_daemon_args,
            memory_scale: node_settings.memory_scale,
        }
    }
}

cfg_if! {

    if #[cfg(not(target_arch = "wasm32"))] {
        impl TryFrom<Config> for Args {
            type Error = Error;
            fn try_from(config: Config) -> Result<Self> {
                let mut args = Args::default();
                match config.network {
                    Network::Mainnet => {}
                    Network::Testnet10 => {
                        args.testnet = true;
                        args.testnet_suffix = 10;
                    }
                    Network::Testnet11 => {
                        args.testnet = true;
                        args.testnet_suffix = 11;
                    }
                }

                args.perf_metrics = true;
                args.perf_metrics_interval_sec = 1;
                args.yes = true;
                args.utxoindex = true;
                args.disable_upnp = !config.enable_upnp;

                if config.enable_grpc {
                    args.rpclisten = Some(config.grpc_network_interface.into());
                }

                args.user_agent_comments = vec![user_agent_comment()];

                // TODO - parse custom args and overlap on top of the defaults

                Ok(args)
            }
        }

        impl From<Config> for Vec<String> {
            fn from(config: Config) -> Self {
                let mut args = Arglist::default();

                match config.network {
                    Network::Mainnet => {}
                    Network::Testnet10 => {
                        args.push("--testnet");
                        args.push("--netsuffix=10");
                    }
                    Network::Testnet11 => {
                        args.push("--testnet");
                        args.push("--netsuffix=11");
                    }
                }

                args.push("--perf-metrics");
                args.push("--perf-metrics-interval-sec=1");
                args.push("--yes");
                args.push("--utxoindex");

                match config.memory_scale {
                    NodeMemoryScale::Default => {},
                    _ => {
                        args.push(format!("--ram-scale={:1.2}", config.memory_scale.get()));
                    }
                }

                if !config.enable_upnp {
                    args.push("--disable-upnp");
                }

                if config.enable_grpc {
                    args.push(format!("--rpclisten={}", config.grpc_network_interface));
                } else {
                    args.push("--nogrpc");
                }

                args.push("--rpclisten-borsh=default");

                args.push(format!("--uacomment={}", user_agent_comment()));

                if config.kaspad_daemon_args_enable {
                    config.kaspad_daemon_args.trim().split(' ').filter(|arg|!arg.trim().is_empty()).for_each(|arg| {
                        args.push(arg);
                    });
                }

                args.into()
            }
        }

        impl IntoIterator for Config {
            type Item = String;
            type IntoIter = std::vec::IntoIter<Self::Item>;

            fn into_iter(self) -> Self::IntoIter {
                let args: Vec<String> = self.into();
                args.into_iter()
            }
        }
    }
}
