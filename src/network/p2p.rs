use libp2p::{
    core::upgrade,
    floodsub::{Floodsub, FloodsubEvent, Topic},
    mdns::{Mdns, MdnsEvent},
    mplex,
    noise::{Keypair, NoiseConfig, X25519Spec},
    swarm::{NetworkBehaviourEventProcess, Swarm, SwarmBuilder},
    tcp::TokioTcpConfig,
    NetworkBehaviour, PeerId, Transport,
};
use log::{error, info};
use std::error::Error;
use tokio::sync::mpsc;

// Custom behavior for the OmniTensor network
#[derive(NetworkBehaviour)]
#[behaviour(event_process = true)]
struct OmniTensorBehaviour {
    floodsub: Floodsub,
    mdns: Mdns,
    #[behaviour(ignore)]
    response_sender: mpsc::UnboundedSender<OmniTensorEvent>,
}

// Custom events for the OmniTensor network
enum OmniTensorEvent {
    NewPeer(PeerId),
    ExpiredPeer(PeerId),
    Message(PeerId, Vec<u8>),
}

impl NetworkBehaviourEventProcess<FloodsubEvent> for OmniTensorBehaviour {
    fn inject_event(&mut self, event: FloodsubEvent) {
        if let FloodsubEvent::Message(message) = event {
            if let Err(e) = self
                .response_sender
                .send(OmniTensorEvent::Message(message.source, message.data))
            {
                error!("Error sending message via channel: {:?}", e);
            }
        }
    }
}

impl NetworkBehaviourEventProcess<MdnsEvent> for OmniTensorBehaviour {
    fn inject_event(&mut self, event: MdnsEvent) {
        match event {
            MdnsEvent::Discovered(list) => {
                for (peer_id, _multiaddr) in list {
                    self.floodsub.add_node_to_partial_view(peer_id);
                    if let Err(e) = self.response_sender.send(OmniTensorEvent::NewPeer(peer_id)) {
                        error!("Error sending new peer event: {:?}", e);
                    }
                }
            }
            MdnsEvent::Expired(list) => {
                for (peer_id, _multiaddr) in list {
                    if !self.mdns.has_node(&peer_id) {
                        self.floodsub.remove_node_from_partial_view(&peer_id);
                        if let Err(e) = self.response_sender.send(OmniTensorEvent::ExpiredPeer(peer_id)) {
                            error!("Error sending expired peer event: {:?}", e);
                        }
                    }
                }
            }
        }
    }
}

pub struct P2PNetwork {
    swarm: Swarm<OmniTensorBehaviour>,
    topic: Topic,
}

impl P2PNetwork {
    pub async fn new() -> Result<(Self, mpsc::UnboundedReceiver<OmniTensorEvent>), Box<dyn Error>> {
        let (response_sender, response_rcv) = mpsc::unbounded_channel();

        let id_keys = Keypair::<X25519Spec>::new()
            .into_authentic(&Keypair::generate())
            .expect("Can create keypair");

        let peer_id = PeerId::from(id_keys.public());
        info!("Local peer id: {:?}", peer_id);

        let transport = TokioTcpConfig::new()
            .upgrade(upgrade::Version::V1)
            .authenticate(NoiseConfig::xx(id_keys).into_authenticated())
            .multiplex(mplex::MplexConfig::new())
            .boxed();

        let topic = Topic::new("omnitensor-messages");

        let mut behaviour = OmniTensorBehaviour {
            floodsub: Floodsub::new(peer_id),
            mdns: Mdns::new(Default::default()).await?,
            response_sender,
        };

        behaviour.floodsub.subscribe(topic.clone());

        let swarm = SwarmBuilder::new(transport, behaviour, peer_id)
            .executor(Box::new(|fut| {
                tokio::spawn(fut);
            }))
            .build();

        Ok((
            Self {
                swarm,
                topic,
            },
            response_rcv,
        ))
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn Error>> {
        self.swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

        loop {
            match self.swarm.next().await {
                Some(event) => {
                    if let swarm::SwarmEvent::NewListenAddr { address, .. } = event {
                        info!("Listening on {:?}", address);
                    }
                }
                None => break,
            }
        }

        Ok(())
    }

    pub fn broadcast(&mut self, message: Vec<u8>) {
        self.swarm
            .behaviour_mut()
            .floodsub
            .publish(self.topic.clone(), message);
    }
}