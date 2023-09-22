use std::{
    sync::{mpsc, Arc, Mutex},
    thread::{self, JoinHandle},
};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        // 하나의 consumer만 존재해야 하므로 clone 하는 방법은 올바르지 않고 데이터가 변할 수 있어 좋지 않은 방법임.
        // Arc 타입은 여러 worker들이 receiver를 소유하는 걸 허용해주고
        // Mutex 타입은 한번에 하나의 worker만이 receiver로부터 데이터를 가져가도록 보장함.
        let receiver = Arc::new(Mutex::new(receiver));

        // 벡터의 공간을 미리 할당. 삽입마다 재할당이 일어나는 Vec::new 를 사용할 때 보다 효율을 높일 수 있음
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            // 스레드들을 생성하고 벡터 내에 보관합니다
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static, // 클로저가 인자를 받지 않고 반환값도 없어서 FnOnce뒤에 ()사용
    {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
}

// 아래는 thread::spawn 시그니처 함수
// spawn 함수는 JoinHandler<T>를 반환하는데 여기서 T는 클로저가 반환할 타입이다.
// 지금 web-server 예제의 경우 스레드풀로 전달 된 클로저는 아무것도 반환하고 있지 않으니 여기선 T가 ()이 된다.
// pub fn spawn<F, T>(f: F) -> JoinHandle<T>
//     where
//         F: FnOnce() -> T,
//         F: Send + 'static,
//         T: Send + 'static,
// {
// }

// worker를 구현해보자. 표준 라이브러리 thread::spawn을 이용해 스레드를 생성할 수 있지만 생성 즉시 샐항할 코드를 전달 받도록 되어 있음.
// 하지만 web-server는 스레드를 생성하고 코드를 전달받을 때까지 기다려야 함. 표준라이브러리는 이러한 방법을 지원하지 않아 직접 구현해야함.
// ThreadPool을 생성할 때 일어나는 일을 정의해보자
// 1. id 와 JoinHandle<()> 을 갖는 Woker 구조체 정의
// 2. ThreadPool 을 Worker 인스턴스들의 벡터를 갖도록 변경
// 3. id 숫자를 받은 Worker 인스턴스를 반환하는 Worker::new 함수를 정의. 반환된 Worker인스턴스에서는 id 와 빈 클로저르 생성된 스레드가 포함되어 있음
// 4. ThreadPool::new 내에서 for 루프를 이용해 id를 생성하고 id를 이용해 새 Worker 를 생성한 뒤 해당 워커를 벡터안에 저장

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
    id: usize,
    thread: JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            // 뮤텍스를 얻기 위해 lock 호출.
            // 뮤텍스를 얻고 채널로부터 Job을 얻기 위해 recv 호출
            // receiver 가져오기 위해 move 사용
            while let Ok(job) = receiver.lock().unwrap().recv() {
                println!("Worker {id} got a job; executing.");

                job();
            }
        });
        Worker { id, thread }
    }
}
