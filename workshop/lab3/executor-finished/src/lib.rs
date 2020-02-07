mod delay;
mod executor;

pub use delay::{delay_for, Delay};
pub use executor::{new_runtime, Executor, Spawner, Task};

#[cfg(test)]
mod tests {
    use super::*;

    use std::time::{Duration, Instant};

    use delay::delay_for;

    #[test]
    fn executor_works() {
        let (executor, spawner) = new_runtime();
        spawner.spawn(async move {
            delay_for(Duration::from_secs(1)).await;
        });

        drop(spawner);

        let start = Instant::now();
        executor.run();
        assert!(start.elapsed() >= Duration::from_secs(1));
    }
}
