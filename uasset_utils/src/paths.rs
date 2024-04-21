pub use typed_path::Utf8Component as PakPathComponentTrait;
pub use typed_path::Utf8UnixComponent as PakPathComponent;
pub use typed_path::Utf8UnixPath as PakPath;
pub use typed_path::Utf8UnixPathBuf as PakPathBuf;

pub fn pak_path_to_game_path<P: AsRef<PakPath>>(pak_path: P) -> Option<String> {
    let mut components = pak_path.as_ref().components();
    match components.next() {
        Some(PakPathComponent::Normal("Engine")) => match components.next() {
            Some(PakPathComponent::Normal("Content")) => {
                Some(PakPath::new("/Engine").join(components.as_path()))
            }
            Some(PakPathComponent::Normal("Plugins")) => {
                let mut last = None;
                loop {
                    match components.next() {
                        Some(PakPathComponent::Normal("Content")) => {
                            break last.map(|plugin| {
                                PakPath::new("/").join(plugin).join(components.as_path())
                            })
                        }
                        Some(PakPathComponent::Normal(next)) => {
                            last = Some(next);
                        }
                        _ => break None,
                    }
                }
            }
            _ => None,
        },
        Some(PakPathComponent::Normal(_)) => match components.next() {
            Some(PakPathComponent::Normal("Content")) => {
                Some(PakPath::new("/Game").join(components))
            }
            _ => None,
        },
        _ => None,
    }
    .map(|p| p.to_string())
}
