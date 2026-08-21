use ellastic_errors::{
  EllasticError,
  Result,
};
use std::time::{
  Duration,
  Instant,
  SystemTime,
  UNIX_EPOCH,
};

#[derive(Debug, Clone)]
pub struct Stopwatch {
  start_time: Instant,
  end_time: Option<Instant>,
  running: bool,
}

impl Stopwatch {
  pub fn new() -> Self {
    Self {
      start_time: Instant::now(),
      end_time: None,
      running: true,
    }
  }

  pub fn start() -> Self {
    Self::new()
  }

  pub fn start_new() -> Self {
    Self {
      start_time: Instant::now(),
      end_time: None,
      running: true,
    }
  }

  pub fn restart(&mut self) {
    self.start_time = Instant::now();
    self.end_time = None;
    self.running = true;
  }

  pub fn stop(&mut self) {
    if self.running {
      self.end_time = Some(Instant::now());
      self.running = false;
    }
  }

  pub fn reset(&mut self) {
    self.start_time = Instant::now();
    self.end_time = None;
    self.running = true;
  }

  pub fn elapsed(&self) -> Duration {
    if let Some(end_time) = self.end_time {
      end_time.duration_since(self.start_time)
    } else {
      self.start_time.elapsed()
    }
  }

  pub fn elapsed_ms(&self) -> u64 {
    self.elapsed().as_millis() as u64
  }

  pub fn elapsed_seconds(&self) -> f64 {
    self.elapsed().as_secs_f64()
  }

  pub fn is_running(&self) -> bool {
    self.running
  }

  pub fn lap(&mut self) -> Duration {
    let now = Instant::now();
    let elapsed = now.duration_since(self.start_time);
    self.start_time = now;
    elapsed
  }

  pub fn lap_ms(&mut self) -> u64 {
    self.lap().as_millis() as u64
  }
}

impl Default for Stopwatch {
  fn default() -> Self {
    Self::new()
  }
}

#[derive(Debug, Clone)]
pub struct Timer {
  duration: Duration,
  start_time: Instant,
  completed: bool,
}

impl Timer {
  pub fn new(duration: Duration) -> Self {
    Self {
      duration,
      start_time: Instant::now(),
      completed: false,
    }
  }

  pub fn from_millis(millis: u64) -> Self {
    Self::new(Duration::from_millis(millis))
  }

  pub fn from_seconds(seconds: u64) -> Self {
    Self::new(Duration::from_secs(seconds))
  }

  pub fn from_minutes(minutes: u64) -> Self {
    Self::new(Duration::from_secs(minutes * 60))
  }

  pub fn from_hours(hours: u64) -> Self {
    Self::new(Duration::from_secs(hours * 3600))
  }

  pub fn reset(&mut self) {
    self.start_time = Instant::now();
    self.completed = false;
  }

  pub fn elapsed(&self) -> Duration {
    self.start_time.elapsed()
  }

  pub fn remaining(&self) -> Duration {
    if self.is_expired() {
      Duration::ZERO
    } else {
      self.duration.saturating_sub(self.elapsed())
    }
  }

  pub fn is_expired(&self) -> bool {
    self.elapsed() >= self.duration
  }

  pub fn progress(&self) -> f64 {
    let elapsed = self.elapsed();
    if elapsed >= self.duration {
      1.0
    } else {
      elapsed.as_secs_f64() / self.duration.as_secs_f64()
    }
  }

  pub fn progress_percentage(&self) -> f64 {
    self.progress() * 100.0
  }

  pub fn wait(&self) {
    if !self.is_expired() {
      std::thread::sleep(self.remaining());
    }
  }

  pub fn wait_async(&self) -> tokio::time::Sleep {
    tokio::time::sleep(self.remaining())
  }
}

#[derive(Debug, Clone)]
pub struct RateLimiter {
  interval: Duration,
  last_reset: Instant,
  tokens: u32,
  max_tokens: u32,
}

impl RateLimiter {
  pub fn new(max_tokens: u32, interval: Duration) -> Self {
    Self {
      interval,
      last_reset: Instant::now(),
      tokens: max_tokens,
      max_tokens,
    }
  }

  pub fn per_second(max_tokens: u32) -> Self {
    Self::new(max_tokens, Duration::from_secs(1))
  }

  pub fn per_minute(max_tokens: u32) -> Self {
    Self::new(max_tokens, Duration::from_secs(60))
  }

  pub fn per_hour(max_tokens: u32) -> Self {
    Self::new(max_tokens, Duration::from_secs(3600))
  }

  pub fn try_acquire(&mut self) -> bool {
    self.reset_if_needed();

    if self.tokens > 0 {
      self.tokens -= 1;
      true
    } else {
      false
    }
  }

  pub fn acquire(&mut self) -> Result<()> {
    while !self.try_acquire() {
      std::thread::sleep(Duration::from_millis(10));
    }
    Ok(())
  }

  pub async fn acquire_async(&mut self) -> Result<()> {
    while !self.try_acquire() {
      tokio::time::sleep(Duration::from_millis(10)).await;
    }
    Ok(())
  }

  pub fn available_tokens(&self) -> u32 {
    self.tokens
  }

  pub fn max_tokens(&self) -> u32 {
    self.max_tokens
  }

  pub fn reset(&mut self) {
    self.last_reset = Instant::now();
    self.tokens = self.max_tokens;
  }

  fn reset_if_needed(&mut self) {
    if self.last_reset.elapsed() >= self.interval {
      self.last_reset = Instant::now();
      self.tokens = self.max_tokens;
    }
  }

  pub fn time_until_next_token(&self) -> Duration {
    if self.tokens > 0 {
      Duration::ZERO
    } else {
      let elapsed = self.last_reset.elapsed();
      if elapsed >= self.interval {
        Duration::ZERO
      } else {
        self.interval - elapsed
      }
    }
  }
}

#[derive(Debug, Clone)]
pub struct Debouncer {
  delay: Duration,
  last_call: Instant,
  pending_timer: Option<Timer>,
}

impl Debouncer {
  pub fn new(delay: Duration) -> Self {
    Self {
      delay,
      last_call: Instant::now(),
      pending_timer: None,
    }
  }

  pub fn from_millis(millis: u64) -> Self {
    Self::new(Duration::from_millis(millis))
  }

  pub fn from_seconds(seconds: u64) -> Self {
    Self::new(Duration::from_secs(seconds))
  }

  pub fn should_execute(&mut self) -> bool {
    let now = Instant::now();
    let elapsed = now.duration_since(self.last_call);

    if elapsed >= self.delay {
      self.last_call = now;
      true
    } else {
      if self.pending_timer.is_none() {
        self.pending_timer = Some(Timer::new(self.delay - elapsed));
      }
      false
    }
  }

  pub fn execute<F, R>(&mut self, f: F) -> Option<R>
  where
    F: FnOnce() -> R,
  {
    if self.should_execute() {
      Some(f())
    } else {
      None
    }
  }

  pub async fn execute_async<F, Fut, R>(&mut self, f: F) -> Option<R>
  where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = R>,
  {
    if self.should_execute() {
      Some(f().await)
    } else {
      if let Some(ref timer) = self.pending_timer {
        timer.wait_async().await;
        self.pending_timer = None;
        if self.should_execute() {
          return Some(f().await);
        }
      }
      None
    }
  }

  pub fn reset(&mut self) {
    self.last_call = Instant::now();
    self.pending_timer = None;
  }

  pub fn time_until_next_execution(&self) -> Duration {
    let elapsed = self.last_call.elapsed();
    if elapsed >= self.delay {
      Duration::ZERO
    } else {
      self.delay - elapsed
    }
  }
}

#[derive(Debug, Clone)]
pub struct Throttler {
  min_interval: Duration,
  last_execution: Instant,
}

impl Throttler {
  pub fn new(min_interval: Duration) -> Self {
    Self {
      min_interval,
      last_execution: Instant::now(),
    }
  }

  pub fn from_millis(millis: u64) -> Self {
    Self::new(Duration::from_millis(millis))
  }

  pub fn from_seconds(seconds: u64) -> Self {
    Self::new(Duration::from_secs(seconds))
  }

  pub fn try_execute<F, R>(&mut self, f: F) -> Option<R>
  where
    F: FnOnce() -> R,
  {
    let now = Instant::now();
    let elapsed = now.duration_since(self.last_execution);

    if elapsed >= self.min_interval {
      self.last_execution = now;
      Some(f())
    } else {
      None
    }
  }

  pub fn execute<F, R>(&mut self, f: F) -> R
  where
    F: FnOnce() -> R,
  {
    let now = Instant::now();
    let elapsed = now.duration_since(self.last_execution);

    if elapsed < self.min_interval {
      std::thread::sleep(self.min_interval - elapsed);
    }

    self.last_execution = Instant::now();
    f()
  }

  pub async fn execute_async<F, Fut, R>(&mut self, f: F) -> R
  where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = R>,
  {
    let now = Instant::now();
    let elapsed = now.duration_since(self.last_execution);

    if elapsed < self.min_interval {
      tokio::time::sleep(self.min_interval - elapsed).await;
    }

    self.last_execution = Instant::now();
    f().await
  }

  pub fn reset(&mut self) {
    self.last_execution = Instant::now();
  }

  pub fn time_until_next_execution(&self) -> Duration {
    let elapsed = self.last_execution.elapsed();
    if elapsed >= self.min_interval {
      Duration::ZERO
    } else {
      self.min_interval - elapsed
    }
  }
}

#[derive(Debug, Clone)]
pub struct TimeWindow {
  window_duration: Duration,
  samples: Vec<(Instant, f64)>,
}

impl TimeWindow {
  pub fn new(window_duration: Duration) -> Self {
    Self {
      window_duration,
      samples: Vec::new(),
    }
  }

  pub fn from_seconds(seconds: u64) -> Self {
    Self::new(Duration::from_secs(seconds))
  }

  pub fn from_minutes(minutes: u64) -> Self {
    Self::new(Duration::from_secs(minutes * 60))
  }

  pub fn add_sample(&mut self, value: f64) {
    let now = Instant::now();
    self.samples.push((now, value));
    self.cleanup_old_samples();
  }

  pub fn average(&self) -> Option<f64> {
    if self.samples.is_empty() {
      None
    } else {
      let sum: f64 = self.samples.iter().map(|(_, value)| *value).sum();
      Some(sum / self.samples.len() as f64)
    }
  }

  pub fn sum(&self) -> Option<f64> {
    if self.samples.is_empty() {
      None
    } else {
      Some(self.samples.iter().map(|(_, value)| *value).sum())
    }
  }

  pub fn count(&self) -> usize {
    self.samples.len()
  }

  pub fn min(&self) -> Option<f64> {
    self.samples.iter().map(|(_, value)| *value).reduce(f64::min)
  }

  pub fn max(&self) -> Option<f64> {
    self.samples.iter().map(|(_, value)| *value).reduce(f64::max)
  }

  pub fn rate(&self) -> Option<f64> {
    if self.samples.is_empty() {
      None
    } else {
      Some(self.samples.len() as f64 / self.window_duration.as_secs_f64())
    }
  }

  pub fn clear(&mut self) {
    self.samples.clear();
  }

  fn cleanup_old_samples(&mut self) {
    let cutoff = Instant::now() - self.window_duration;
    self.samples.retain(|(timestamp, _)| *timestamp >= cutoff);
  }

  pub fn window_duration(&self) -> Duration {
    self.window_duration
  }
}

#[derive(Debug, Clone)]
pub struct PerformanceTimer {
  name: String,
  stopwatch: Stopwatch,
}

impl PerformanceTimer {
  pub fn new(name: String) -> Self {
    Self {
      name,
      stopwatch: Stopwatch::new(),
    }
  }

  pub fn elapsed(&self) -> Duration {
    self.stopwatch.elapsed()
  }

  pub fn elapsed_ms(&self) -> u64 {
    self.stopwatch.elapsed_ms()
  }

  pub fn stop(&mut self) {
    self.stopwatch.stop();
    tracing::debug!(
      "Performance timer '{}' stopped: {}ms",
      self.name,
      self.elapsed_ms()
    );
  }

  pub fn lap(&mut self) -> Duration {
    let lap_time = self.stopwatch.lap();
    tracing::debug!(
      "Performance timer '{}' lap: {}ms",
      self.name,
      lap_time.as_millis()
    );
    lap_time
  }

  pub fn lap_ms(&mut self) -> u64 {
    self.lap().as_millis() as u64
  }
}

impl Drop for PerformanceTimer {
  fn drop(&mut self) {
    if self.stopwatch.is_running() {
      self.stop();
    }
  }
}

pub fn create_stopwatch() -> Stopwatch {
  Stopwatch::new()
}

pub fn create_timer(duration: Duration) -> Timer {
  Timer::new(duration)
}

pub fn create_timer_millis(millis: u64) -> Timer {
  Timer::from_millis(millis)
}

pub fn create_timer_seconds(seconds: u64) -> Timer {
  Timer::from_seconds(seconds)
}

pub fn create_rate_limiter(max_tokens: u32, interval: Duration) -> RateLimiter {
  RateLimiter::new(max_tokens, interval)
}

pub fn create_rate_limiter_per_second(max_tokens: u32) -> RateLimiter {
  RateLimiter::per_second(max_tokens)
}

pub fn create_debouncer(delay: Duration) -> Debouncer {
  Debouncer::new(delay)
}

pub fn create_throttler(min_interval: Duration) -> Throttler {
  Throttler::new(min_interval)
}

pub fn create_time_window(window_duration: Duration) -> TimeWindow {
  TimeWindow::new(window_duration)
}

pub fn create_performance_timer(name: String) -> PerformanceTimer {
  PerformanceTimer::new(name)
}

pub fn current_timestamp() -> u64 {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap_or_default()
    .as_secs()
}

pub fn current_timestamp_millis() -> u64 {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap_or_default()
    .as_millis() as u64
}

pub fn format_duration(duration: Duration) -> String {
  let total_seconds = duration.as_secs();
  let hours = total_seconds / 3600;
  let minutes = (total_seconds % 3600) / 60;
  let seconds = total_seconds % 60;
  let millis = duration.subsec_millis();

  if hours > 0 {
    format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, millis)
  } else if minutes > 0 {
    format!("{:02}:{:02}.{:03}", minutes, seconds, millis)
  } else {
    format!("{:02}.{:03}s", seconds, millis)
  }
}

pub fn parse_duration(duration_str: &str) -> Result<Duration> {
  let duration_str = duration_str.trim().to_lowercase();

  if duration_str.ends_with("ms") {
    let millis: u64 = duration_str[..duration_str.len() - 2]
      .parse()
      .map_err(|_| EllasticError::InvalidParameter("Invalid duration format".to_string()))?;
    Ok(Duration::from_millis(millis))
  } else if duration_str.ends_with('s') {
    let seconds: u64 = duration_str[..duration_str.len() - 1]
      .parse()
      .map_err(|_| EllasticError::InvalidParameter("Invalid duration format".to_string()))?;
    Ok(Duration::from_secs(seconds))
  } else if duration_str.ends_with('m') {
    let minutes: u64 = duration_str[..duration_str.len() - 1]
      .parse()
      .map_err(|_| EllasticError::InvalidParameter("Invalid duration format".to_string()))?;
    Ok(Duration::from_secs(minutes * 60))
  } else if duration_str.ends_with('h') {
    let hours: u64 = duration_str[..duration_str.len() - 1]
      .parse()
      .map_err(|_| EllasticError::InvalidParameter("Invalid duration format".to_string()))?;
    Ok(Duration::from_secs(hours * 3600))
  } else {
    let seconds: u64 = duration_str
      .parse()
      .map_err(|_| EllasticError::InvalidParameter("Invalid duration format".to_string()))?;
    Ok(Duration::from_secs(seconds))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_stopwatch() {
    let mut stopwatch = Stopwatch::new();
    std::thread::sleep(Duration::from_millis(10));

    let elapsed = stopwatch.elapsed();
    assert!(elapsed >= Duration::from_millis(10));

    stopwatch.stop();
    assert!(!stopwatch.is_running());
  }

  #[test]
  fn test_timer() {
    let timer = Timer::from_millis(10);
    assert!(!timer.is_expired());

    std::thread::sleep(Duration::from_millis(15));
    assert!(timer.is_expired());
  }

  #[test]
  fn test_rate_limiter() {
    let mut rate_limiter = RateLimiter::new(2, Duration::from_millis(100));

    assert!(rate_limiter.try_acquire());
    assert!(rate_limiter.try_acquire());
    assert!(!rate_limiter.try_acquire());

    std::thread::sleep(Duration::from_millis(110));
    assert!(rate_limiter.try_acquire());
  }

  #[test]
  fn test_debouncer() {
    let mut debouncer = Debouncer::from_millis(50);

    assert!(debouncer.should_execute());
    assert!(!debouncer.should_execute());

    std::thread::sleep(Duration::from_millis(60));
    assert!(debouncer.should_execute());
  }

  #[test]
  fn test_throttler() {
    let mut throttler = Throttler::from_millis(50);

    let result1 = throttler.try_execute(|| 1);
    assert!(result1.is_some());

    let result2 = throttler.try_execute(|| 2);
    assert!(result2.is_none());

    std::thread::sleep(Duration::from_millis(60));
    let result3 = throttler.try_execute(|| 3);
    assert!(result3.is_some());
  }

  #[test]
  fn test_time_window() {
    let mut window = TimeWindow::from_seconds(1);

    window.add_sample(1.0);
    window.add_sample(2.0);
    window.add_sample(3.0);

    assert_eq!(window.count(), 3);
    assert_eq!(window.average(), Some(2.0));
    assert_eq!(window.sum(), Some(6.0));
    assert_eq!(window.min(), Some(1.0));
    assert_eq!(window.max(), Some(3.0));
  }

  #[test]
  fn test_format_duration() {
    let duration = Duration::from_millis(1500);
    assert_eq!(format_duration(duration), "01.500s");

    let duration = Duration::from_secs(65) + Duration::from_millis(500);
    assert_eq!(format_duration(duration), "01:05.500");

    let duration = Duration::from_secs(3661) + Duration::from_millis(500);
    assert_eq!(format_duration(duration), "01:01:01.500");
  }

  #[test]
  fn test_parse_duration() {
    assert_eq!(parse_duration("100ms").unwrap(), Duration::from_millis(100));
    assert_eq!(parse_duration("10s").unwrap(), Duration::from_secs(10));
    assert_eq!(parse_duration("5m").unwrap(), Duration::from_secs(300));
    assert_eq!(parse_duration("2h").unwrap(), Duration::from_secs(7200));
    assert_eq!(parse_duration("30").unwrap(), Duration::from_secs(30));
  }
}
