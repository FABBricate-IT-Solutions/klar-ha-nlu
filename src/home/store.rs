use crate::types::HomeGraph;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct HomeStore {
    inner: Arc<Mutex<Arc<HomeGraph>>>,
}

impl HomeStore {
    pub fn new(graph: HomeGraph) -> Self {
        Self { inner: Arc::new(Mutex::new(Arc::new(graph))) }
    }

    pub async fn snapshot(&self) -> Arc<HomeGraph> {
        self.inner.lock().await.clone()
    }

    pub async fn replace(&self, graph: HomeGraph) {
        *self.inner.lock().await = Arc::new(graph);
    }

    pub async fn edit<T>(&self, edit: impl FnOnce(&mut HomeGraph) -> T) -> T {
        let mut guard = self.inner.lock().await;
        let mut next = (**guard).clone();
        let out = edit(&mut next);
        *guard = Arc::new(next);
        out
    }
}
