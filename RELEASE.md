# Release

## Update Dependencies
 * ```cargo update```

## Automated Testing

 * ```cargo test```
 * ```cargo clippy```
 * ```cargo publish --dry-run --allow-dirty```

## Manual Testing

 * ```cargo run``` in a terminal
  * Double-check the dates, duration, remaining, etc.
  * Resize the window.  It should update in 10s.
  * From 10m to 9m it should not display a rendering artifact.
  * CTRL-C to quit.
 * ```cargo run -- --task "hello world"``` in a terminal  
  * It should show the task.

## Commit & Push

 * Commit changes
 * Push changes to master
 * ```cargo publish```