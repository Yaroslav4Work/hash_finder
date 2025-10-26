pub mod sync_impl {
    use sha256::digest;

    pub struct HashFinder {
        n: u8,
        f: u32,
    }

    impl HashFinder {
        pub fn new(n: u8, f: u32) -> HashFinder {
            HashFinder { n, f }
        }

        pub fn run(&self) {
            let mut matches = 0;
            let mut counter = 0;

            loop {
                let hash = digest(counter.to_string());

                if hash.ends_with(&"0".repeat(self.n as usize)) {
                    if matches == self.f {
                        break;
                    }

                    println!("{}, {}", counter, hash);

                    matches += 1;
                }

                counter += 1;
            }
        }
    }
}

pub struct EventData {
    hash: String,
    number: u32,
}

pub mod async_impl {
    use crate::EventData;
    use crate::async_impl::ChannelError::IncorrectChannelEventVariant;
    use sha256::digest;
    use std::collections::HashMap;
    use thiserror::Error;
    use tokio::spawn;
    use tokio::sync::mpsc::error::SendError;
    use tokio::sync::mpsc::{Sender, channel};
    use tokio::task::{JoinError, JoinSet};

    pub enum ChannelEvent {
        FromChildTask {
            thread_id: u8,
            data: Option<EventData>,
        },
        FromControlTask(Option<u32>),
    }

    #[derive(Error, Debug)]
    pub enum ChannelError {
        #[error("Incorrect channel event variant")]
        IncorrectChannelEventVariant,
        #[error("Channel sending error")]
        SendError(#[from] SendError<ChannelEvent>),
    }

    pub struct HashFinder {
        tasks_count: u8,
        n: u8,
        f: u32,
    }

    impl HashFinder {
        pub fn new(tasks_count: u8, n: u8, f: u32) -> HashFinder {
            HashFinder { tasks_count, n, f }
        }

        pub async fn run(&self) {
            let mut from_main_senders = HashMap::<u8, Sender<ChannelEvent>>::new();
            let mut from_thread_receivers = Vec::new();

            let mut async_tasks = Vec::new();

            for i in 0..self.tasks_count {
                let (m_sender, mut m_receiver) = channel::<ChannelEvent>(1024);
                let (t_sender, t_receiver) = channel::<ChannelEvent>(1024);

                from_main_senders.insert(i, m_sender);
                from_thread_receivers.push(t_receiver);

                let n = self.n.clone() as usize;

                let thread = spawn(async move {
                    loop {
                        if let Ok(event) = m_receiver.try_recv() {
                            let number = match event {
                                ChannelEvent::FromChildTask { .. } => {
                                    return Err(IncorrectChannelEventVariant);
                                }
                                ChannelEvent::FromControlTask(number) => number,
                            };

                            let number = match number {
                                Some(n) => n,
                                // If number is None it's expected behavior - end task
                                None => return Ok(()),
                            };

                            let hash = digest(number.to_string());

                            let mut data = None;

                            if hash.ends_with(&"0".repeat(n)) {
                                data = Some(EventData { hash, number });
                            }

                            // Channel can be closed already - it's normal behavior
                            let _ = t_sender
                                .send(ChannelEvent::FromChildTask { thread_id: i, data })
                                .await?;
                        }
                    }
                });

                async_tasks.push(thread);
            }

            let f = self.f.clone();
            let tasks_count = self.tasks_count.clone();

            let control_task = spawn(async move {
                let mut counter = 0;
                let mut number = 0;

                for i in 0..tasks_count {
                    from_main_senders[&i]
                        .send(ChannelEvent::FromControlTask(Some(number)))
                        .await?;
                    number += 1;
                }

                while counter < f {
                    for i in &mut from_thread_receivers {
                        while let Ok(event) = i.try_recv() {
                            let (thread_id, data) = match event {
                                ChannelEvent::FromChildTask { thread_id, data } => {
                                    (thread_id, data)
                                }
                                ChannelEvent::FromControlTask(_) => {
                                    return Err(IncorrectChannelEventVariant);
                                }
                            };

                            if let Some(data) = data {
                                println!("{}, {}", data.number, data.hash);
                                counter += 1;
                            }

                            number += 1;

                            from_main_senders[&thread_id]
                                .send(ChannelEvent::FromControlTask(Some(number)))
                                .await?;
                        }
                    }
                }

                for i in 0..tasks_count {
                    from_main_senders[&i]
                        .send(ChannelEvent::FromControlTask(None))
                        .await?;
                }

                Ok(())
            });

            async_tasks.push(control_task);

            let res = async_tasks
                .into_iter()
                .collect::<JoinSet<Result<Result<(), ChannelError>, JoinError>>>()
                .join_all()
                .await;
            
            for i in res {
                if let Err(join_err) = i {
                    eprintln!("{:?}", join_err);
                }
            }
        }
    }
}

pub mod multithread_impl {
    use crate::EventData;
    use sha256::digest;
    use std::collections::HashMap;
    use std::sync::mpsc::{SendError, Sender, channel};
    use std::thread::spawn;
    use thiserror::Error;
    use tokio::task::JoinError;

    pub enum ChannelEvent {
        FromChildTask {
            thread_id: u8,
            data: Option<EventData>,
        },
        FromControlTask(Option<u32>),
    }

    #[derive(Error, Debug)]
    pub enum HashFinderError {
        #[error("Incorrect channel event variant")]
        IncorrectChannelEventVariant,
        #[error("Channel sending error")]
        SendError(#[from] SendError<ChannelEvent>),
        #[error("Join handle error")]
        JoinError(#[from] JoinError),
    }

    pub struct HashFinder {
        threads_count: u8,
        n: u8,
        f: u32,
    }

    impl HashFinder {
        pub fn new(threads_count: u8, n: u8, f: u32) -> HashFinder {
            HashFinder {
                threads_count,
                n,
                f,
            }
        }

        pub fn run(&self) -> Result<(), HashFinderError> {
            let mut from_main_senders = HashMap::<u8, Sender<ChannelEvent>>::new();
            let mut from_thread_receivers = Vec::new();

            let mut threads = Vec::new();

            for i in 0..self.threads_count {
                let (m_sender, m_receiver) = channel::<ChannelEvent>();
                let (t_sender, t_receiver) = channel::<ChannelEvent>();

                from_main_senders.insert(i, m_sender);
                from_thread_receivers.push(t_receiver);

                let n = self.n.clone() as usize;

                let thread = spawn(move || {
                    loop {
                        if let Ok(event) = m_receiver.try_recv() {
                            let number = match event {
                                ChannelEvent::FromChildTask { .. } => {
                                    return Err(HashFinderError::IncorrectChannelEventVariant);
                                }
                                ChannelEvent::FromControlTask(number) => number,
                            };

                            let number = match number {
                                Some(n) => n,
                                // If number is None it's expected behavior - end task
                                None => return Ok(()),
                            };

                            let hash = digest(number.to_string());

                            let mut data = None;

                            if hash.ends_with(&"0".repeat(n)) {
                                data = Some(EventData { hash, number });
                            }

                            t_sender.send(ChannelEvent::FromChildTask { thread_id: i, data })?;
                        }
                    }
                });

                threads.push(thread);
            }

            let f = self.f.clone();
            let threads_count = self.threads_count.clone();

            threads.push(spawn(move || {
                let mut counter = 0;
                let mut number = 0;

                for i in 0..threads_count {
                    from_main_senders[&i].send(ChannelEvent::FromControlTask(Some(number)))?;
                    number += 1;
                }

                while counter < f {
                    for i in &from_thread_receivers {
                        while let Ok(event) = i.try_recv() {
                            let (thread_id, data) = match event {
                                ChannelEvent::FromChildTask { thread_id, data } => {
                                    (thread_id, data)
                                }
                                ChannelEvent::FromControlTask(_) => {
                                    return Err(HashFinderError::IncorrectChannelEventVariant);
                                }
                            };

                            if let Some(event) = data {
                                println!("{}, {}", event.number, event.hash);
                                counter += 1;
                            }

                            number += 1;

                            from_main_senders[&thread_id]
                                .send(ChannelEvent::FromControlTask(Some(number)))?;
                        }
                    }
                }

                for i in 0..threads_count {
                    from_main_senders[&i].send(ChannelEvent::FromControlTask(None))?;
                }

                Ok(())
            }));

            for thread in threads {
                thread
                    .join()
                    .expect("Unexpected error. Failed to join thread")?;
            }

            Ok(())
        }
    }
}
