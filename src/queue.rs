#![allow(dead_code)]
#![allow(unused_variables)]

use std::any::Any;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

macro_rules! drop_and_sleep {
    (
        drop: [$($d:expr),*],
        sleep: $s:expr
    ) => {
        let delay = $s.clone();
        $(
          std::mem::drop($d);
        )*
        std::thread::sleep(delay);
    }
}

pub trait QueueJob {
    fn run(&mut self);
}

pub struct QueueData<T> {
    jobs: Vec<T>,
    /// The number of jobs that can run in parallel before a new job starts.
    simultaneously: usize,
    /// The delay between individual jobs
    /// Warning, this is a natural delay.
    /// It does not wait until existing jobs are finished.
    /// In the end the 'delay' consists of 'delay' + 'sampling_rate'
    delay: Duration,
    /// The sampling rate determines how often new entries in the queue are asked for.
    /// In doing so, all variables borrowed from the thread are unlocked
    /// and a natural delay is invoked.
    sampling_rate: Duration,
    /// It is possible to cause a delay when a certain number of jobs have already been processed.
    /// It can be used to ensure a forced pause when too many requests are made.
    job_limit_delay: Option<Duration>,
    /// The number before the limit delay kicks in.
    job_limit: usize,
}

impl<T> QueueData<T>
where
    T: QueueJob + Send,
{
    pub fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            jobs: Vec::<T>::new(),
            simultaneously: 2,
            delay: Duration::from_secs(2),
            sampling_rate: Duration::from_secs(2),
            job_limit_delay: None,
            job_limit: 0,
        }))
    }
}

pub struct Queue<T> {
    data: Arc<Mutex<QueueData<T>>>,
    main_job_handle: Option<JoinHandle<()>>,
    job_handles: Arc<Mutex<Vec<JoinHandle<()>>>>,
}

impl<T> Queue<T>
where
    T: QueueJob + Send + 'static,
{
    pub fn new() -> Self {
        Queue {
            data: QueueData::new(),
            job_handles: Arc::new(Mutex::new(Vec::new())),
            main_job_handle: None,
        }
    }

    pub fn join(self) -> Result<(), Box<dyn Any + Send + 'static>> {
        self.main_job_handle.unwrap().join()
    }

    pub fn push(&mut self, job: T) {
        self.data.lock().unwrap().jobs.insert(0, job);
    }

    pub fn schedule(&mut self) {
        let jobs = Arc::clone(&self.data);
        let job_handles = Arc::clone(&self.job_handles);
        let delay_counter: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));

        let job_handle = thread::spawn(move || {
            loop {
                let mut data = jobs.lock().unwrap();
                let mut counter = delay_counter.lock().unwrap();
                let mut job_handles = job_handles.lock().unwrap();
                if data.jobs.len() < 1 {
                    drop_and_sleep!(drop: [data, counter, job_handles], sleep: &data.sampling_rate);
                    continue;
                }

                if let Some(value) = &data.job_limit_delay {
                    if &*counter > &data.job_limit {
                        *counter = 0;
                        drop_and_sleep!(drop: [], sleep: value);
                    }
                }

                // Clean up old thread handles
                job_handles.retain(|handle| handle.is_finished() == false);

                if &job_handles.len() > &data.simultaneously {
                    drop_and_sleep!(drop: [data, counter, job_handles], sleep: &data.sampling_rate);
                    continue;
                }

                let earliest_job = data.jobs.pop();
                if let Some(value) = earliest_job {
                    drop_and_sleep!(drop: [], sleep: &data.delay);
                    job_handles.push(generate_job_thread(value));
                    *counter += 1;
                }
            }
        });

        self.main_job_handle = Some(job_handle);
    }
}

fn generate_job_thread<T>(mut job: T) -> JoinHandle<()>
where
    T: QueueJob + Send + 'static,
{
    thread::spawn(move || {
        job.run();
    })
}
