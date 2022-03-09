use crate::mux::tab::Tab;
use crate::pty::{unix, PtySize, PtySystem};
use std::cell::{Ref, RefCell};
use std::io::Read;
use std::process::Command;
use std::rc::Rc;
use std::thread;
use ratelimit_meter::algorithms::NonConformance;
use ratelimit_meter::{DirectRateLimiter, LeakyBucket, NegativeMultiDecision};

pub mod tab;

pub struct Mux {
    tab: RefCell<Tab>,
}

pub struct RateLimiter {
    lim: DirectRateLimiter<LeakyBucket>,
}

impl RateLimiter {
    pub fn new(capacity_per_second: u32) -> Self {
        Self {
            lim: DirectRateLimiter::<LeakyBucket>::per_second(
                std::num::NonZeroU32::new(capacity_per_second)
                    .expect("RateLimiter capacity to be non-zero"),
            ),
        }
    }

    pub fn blocking_admittance_check(&mut self, amount: u32) {
        loop {
            match self.lim.check_n(amount) {
                Ok(_) => return,
                Err(NegativeMultiDecision::BatchNonConforming(_, over)) => {
                    let duration = over.wait_time_from(std::time::Instant::now());
                    std::thread::sleep(duration);
                }
                Err(err) => panic!("{}", err),
            }
        }
    }
}

use async_task::{JoinHandle, Task};
use std::future::Future;
use std::sync::Mutex;

pub type SpawnFunc = Box<dyn FnOnce() + Send>;
pub type ScheduleFunc = Box<dyn Fn(Task<()>) + Send + Sync + 'static>;

fn no_schedule_configured(_: Task<()>) {
    panic!("no scheduler has been configured");
}

lazy_static::lazy_static! {
    static ref ON_MAIN_THREAD: Mutex<ScheduleFunc> = Mutex::new(Box::new(no_schedule_configured));
    static ref ON_MAIN_THREAD_LOW_PRI: Mutex<ScheduleFunc> = Mutex::new(Box::new(no_schedule_configured));
}

pub fn set_schedulers(main: ScheduleFunc, low_pri: ScheduleFunc) {
    *ON_MAIN_THREAD.lock().unwrap() = Box::new(main);
    *ON_MAIN_THREAD_LOW_PRI.lock().unwrap() = Box::new(low_pri);
}

pub fn spawn<F, R>(future: F) -> JoinHandle<R, ()>
where
    F: Future<Output = R> + 'static,
    R: 'static,
{
    let (task, handle) =
        async_task::spawn_local(future, |task| ON_MAIN_THREAD.lock().unwrap()(task), ());
    task.schedule();
    handle
}

pub fn spawn_into_main_thread<F, R>(future: F) -> JoinHandle<R, ()>
where
    F: Future<Output = R> + Send + 'static,
    R: Send + 'static,
{
    let (task, handle) = async_task::spawn(future, |task| ON_MAIN_THREAD.lock().unwrap()(task), ());
    task.schedule();
    handle
}

pub fn spawn_into_main_thread_with_low_priority<F, R>(future: F) -> JoinHandle<R, ()>
where
    F: Future<Output = R> + Send + 'static,
    R: Send + 'static,
{
    let (task, handle) =
        async_task::spawn(future, |task| ON_MAIN_THREAD_LOW_PRI.lock().unwrap()(task), ());
    task.schedule();
    handle
}

fn read_from_tab_pty(mut reader: Box<dyn std::io::Read>) {
    const BUFSIZE: usize = 32 * 1024;
    let mut buf = [0; BUFSIZE];

    let mut lim =
        RateLimiter::new(2 * 1024 * 1024);

    loop {
        match reader.read(&mut buf) {
            Ok(size) if size == 0 => {
                break;
            }
            Err(_) => {
                break;
            }
            Ok(size) => {
                lim.blocking_admittance_check(size as u32);
                let data = buf[0..size].to_vec();
                println!("{:?}", data);
                spawn_into_main_thread_with_low_priority(async move {
                    let mux = Mux::get().unwrap();
                    let tab = mux.get_tab();
                });
            }
        }
    }
}

thread_local! {
    static MUX: RefCell<Option<Rc<Mux>>> = RefCell::new(None);
}

impl Mux {
    pub fn new(size: PtySize) -> anyhow::Result<Self> {
        let pty_system = Box::new(unix::UnixPtySystem);
        let pair = pty_system.openpty(size)?;

        println!("Running system shell: {}", crate::pty::get_shell()?);
        let child = pair.slave.spawn_command(Command::new(crate::pty::get_shell()?))?;


        let tab = Tab::new(child, pair.master);

        Ok(Self { tab: RefCell::new(tab) })
    }

    pub fn start(&self) -> anyhow::Result<()> {
        let reader = self.tab.borrow().reader()?;
        thread::spawn(move || read_from_tab_pty(reader));

        Ok(())
    }

    pub fn set_mux(mux: &Rc<Mux>) {
        MUX.with(|m| {
            *m.borrow_mut() = Some(Rc::clone(mux));
        });
    }

    pub fn get() -> Option<Rc<Mux>> {
        let mut res = None;
        MUX.with(|m| {
            if let Some(mux) = &*m.borrow() {
                res = Some(Rc::clone(mux));
            }
        });
        res
    }

    pub fn get_tab(&self) -> Ref<Tab> {
        self.tab.borrow()
    }

    pub fn close(&self) {
        self.tab.borrow_mut().close()
    }

    pub fn can_close(&self) -> bool {
        self.tab.borrow().can_close()
    }
}
