# Benchmarking Methodology Specification
# Part 2: UI Responsiveness, Feature Comparison & Test Environments

**Feature**: Native High-Performance Task Manager  
**Spec Reference**: `../spec.md`  
**Created**: 2025-10-21  
**Status**: Benchmarking Protocol

---

## 3. UI RESPONSIVENESS BENCHMARKS

### 3.1 Process List Display Performance

**Objective**: Measure time to populate process list under varying load (SC-002 validation)

**Test Protocol**:

```powershell
# process-list-display-benchmark.ps1

Write-Host "=== Process List Display Performance ===" -ForegroundColor Cyan

$processCountTests = @(50, 150, 500, 1000, 1500, 2000)
$results = @()

foreach ($targetCount in $processCountTests) {
    Write-Host "`nTesting with $targetCount processes..."

    # Spawn dummy processes to reach target
    $currentCount = (Get-Process).Count
    $dummyProcesses = @()
    $needed = [math]::Max(0, $targetCount - $currentCount)

    for ($i = 0; $i -lt $needed; $i++) {
        $dummyProcesses += Start-Process "notepad.exe" -WindowStyle Hidden -PassThru
    }

    Start-Sleep -Seconds 3  # System stabilization

    # Measure list population time (cold start)
    $times = @()
    for ($run = 1; $run -le 10; $run++) {
        # Kill app if running
        Get-Process "rust-task-manager" -ErrorAction SilentlyContinue | Stop-Process -Force
        Start-Sleep -Seconds 2

        # Start with instrumentation
        $startTime = Get-Date
        $process = Start-Process "rust-task-manager.exe" -ArgumentList "--benchmark-list-display" -PassThru

        # Wait for "list populated" signal
        $populatedTime = Wait-ForBenchmarkSignal -ProcessId $process.Id -Signal "LIST_POPULATED" -TimeoutSeconds 10
        $elapsed = ((Get-Date) - $startTime).TotalMilliseconds

        $times += $elapsed
        $process | Stop-Process -Force
        Start-Sleep -Milliseconds 500
    }

    $avgTime = ($times | Measure-Object -Average).Average
    $actualCount = (Get-Process).Count

    $results += [PSCustomObject]@{
        TargetProcesses = $targetCount
        ActualProcesses = $actualCount
        AvgDisplayTime_ms = [math]::Round($avgTime, 2)
        MinTime_ms = [math]::Round(($times | Measure-Object -Minimum).Minimum, 2)
        P95_ms = [math]::Round(($times | Sort-Object)[9], 2)
    }

    # Cleanup
    $dummyProcesses | Stop-Process -Force -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 2
}

# Display results
$results | Format-Table -AutoSize

# Validation: SC-002 requires <50ms for up to 2048 processes
$maxResult = $results | Where-Object TargetProcesses -eq 2000
if ($maxResult.AvgDisplayTime_ms -lt 50) {
    Write-Host "`n✅ PASS: Process list displays in <50ms even with 2000 processes (SC-002)" -ForegroundColor Green
} else {
    Write-Host "`n❌ FAIL: Process list display exceeds 50ms at scale" -ForegroundColor Red
}

# Scaling analysis
$scaling = @()
for ($i = 1; $i -lt $results.Count; $i++) {
    $prev = $results[$i - 1]
    $curr = $results[$i]
    $scaleFactor = $curr.ActualProcesses / $prev.ActualProcesses
    $timeIncrease = ($curr.AvgDisplayTime_ms - $prev.AvgDisplayTime_ms) / $prev.AvgDisplayTime_ms * 100

    $scaling += [PSCustomObject]@{
        From = $prev.ActualProcesses
        To = $curr.ActualProcesses
        ScaleFactor = [math]::Round($scaleFactor, 2)
        TimeIncrease_Percent = [math]::Round($timeIncrease, 1)
    }
}

Write-Host "`nScaling Analysis:"
$scaling | Format-Table -AutoSize

# Target: Sub-linear scaling (time increase < process count increase)
$avgScaling = ($scaling.TimeIncrease_Percent | Measure-Object -Average).Average
if ($avgScaling -lt 50) {  # Time increases less than 50% of process count increase
    Write-Host "✅ PASS: Sub-linear scaling confirmed" -ForegroundColor Green
} else {
    Write-Host "⚠️  WARNING: Scaling may be linear or worse" -ForegroundColor Yellow
}
```

**Instrumentation in Application**:

```rust
// src/benchmark.rs

#[cfg(feature = "benchmark")]
pub fn benchmark_list_display() {
    let start = Instant::now();

    // Enumerate all processes
    let processes = enumerate_processes();
    let enum_time = start.elapsed();

    // Populate UI list structure
    let list_start = Instant::now();
    populate_list_view(&processes);
    let populate_time = list_start.elapsed();

    // Signal completion
    let total_time = start.elapsed();

    eprintln!("BENCHMARK_LIST_DISPLAY:");
    eprintln!("  process_count: {}", processes.len());
    eprintln!("  enumeration_ms: {}", enum_time.as_millis());
    eprintln!("  populate_ms: {}", populate_time.as_millis());
    eprintln!("  total_ms: {}", total_time.as_millis());

    // Write signal to named pipe
    signal_benchmark_complete("LIST_POPULATED", total_time.as_millis());
}
```

**Success Criteria**:
- ✅ 500 processes: <30ms
- ✅ 1000 processes: <40ms
- ✅ 2000 processes: <50ms (SC-002)
- ✅ Scaling: O(log n) or better
- ✅ Jank-free: No frame drops during population

---

### 3.2 Sort Operation Performance

**Objective**: Measure sort latency for various columns and process counts

**Test Protocol**:

```powershell
# sort-benchmark.ps1

Write-Host "=== Sort Operation Performance ===" -ForegroundColor Cyan

$processCountTests = @(500, 1000, 2000)
$sortColumns = @("Name", "PID", "CPU", "Memory", "Status")
$results = @()

foreach ($processCount in $processCountTests) {
    Write-Host "`nTesting with $processCount processes..."

    # Setup test environment
    Setup-TestProcesses -TargetCount $processCount

    # Launch app
    $process = Start-Process "rust-task-manager.exe" -PassThru
    Start-Sleep -Seconds 5  # Wait for initialization

    foreach ($column in $sortColumns) {
        Write-Host "  Sorting by: $column"

        # Trigger sort via UI automation
        $sortTimes = @()
        for ($i = 0; $i -lt 20; $i++) {
            $startTime = Get-Date

            # Simulate column header click (toggle sort direction)
            Invoke-UIAutomation -ProcessId $process.Id -Action "ClickColumnHeader" -Column $column

            # Wait for sort completion signal
            $sortTime = Wait-ForBenchmarkSignal -ProcessId $process.Id -Signal "SORT_COMPLETE" -TimeoutSeconds 5
            $elapsed = ((Get-Date) - $startTime).TotalMilliseconds

            $sortTimes += $elapsed
            Start-Sleep -Milliseconds 100  # Brief pause between sorts
        }

        $avgTime = ($sortTimes | Measure-Object -Average).Average

        $results += [PSCustomObject]@{
            ProcessCount = $processCount
            Column = $column
            AvgSortTime_ms = [math]::Round($avgTime, 2)
            MinTime_ms = [math]::Round(($sortTimes | Measure-Object -Minimum).Minimum, 2)
            MaxTime_ms = [math]::Round(($sortTimes | Measure-Object -Maximum).Maximum, 2)
        }
    }

    $process | Stop-Process -Force
    Cleanup-TestProcesses
}

# Display results
$results | Format-Table -AutoSize

# Validation: All sorts should complete in <5ms for responsiveness
$maxSortTime = ($results | Measure-Object AvgSortTime_ms -Maximum).Maximum
if ($maxSortTime -lt 5.0) {
    Write-Host "`n✅ PASS: All sort operations <5ms (imperceptible)" -ForegroundColor Green
} elseif ($maxSortTime -lt 16.0) {
    Write-Host "`n⚠️  ACCEPTABLE: Sort times <16ms (one frame)" -ForegroundColor Yellow
} else {
    Write-Host "`n❌ FAIL: Sort operations cause visible lag" -ForegroundColor Red
}

# Column-specific analysis
Write-Host "`nSort Performance by Column (at 2000 processes):"
$results | Where-Object ProcessCount -eq 2000 | Sort-Object AvgSortTime_ms | Format-Table Column, AvgSortTime_ms -AutoSize
```

**Instrumentation**:

```rust
// src/ui/list_view.rs

#[cfg(feature = "benchmark")]
fn sort_column_benchmarked(&mut self, column: Column) {
    let start = Instant::now();

    // Perform sort
    match column {
        Column::Name => self.processes.sort_by(|a, b| a.name.cmp(&b.name)),
        Column::Pid => self.processes.sort_by_key(|p| p.pid),
        Column::Cpu => self.processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap()),
        Column::Memory => self.processes.sort_by_key(|p| p.memory_working_set),
        // ... other columns
    }

    let sort_time = start.elapsed();

    // Invalidate and redraw
    self.invalidate();
    let redraw_time = start.elapsed();

    eprintln!("SORT_COMPLETE: column={:?}, sort_ms={}, redraw_ms={}, total_ms={}",
              column, sort_time.as_millis(), (redraw_time - sort_time).as_millis(), redraw_time.as_millis());

    signal_benchmark_complete("SORT_COMPLETE", redraw_time.as_millis());
}
```

**Success Criteria**:
- ✅ Any column, any process count: <5ms (imperceptible)
- ✅ At 2000 processes: <10ms maximum
- ✅ No frame drops during sort animation
- ✅ Consistent performance across all data types (string, int, float)

---

### 3.3 Filter Operation Latency

**Objective**: Measure real-time filter responsiveness (FR-005, <100ms)

**Test Protocol**:

```powershell
# filter-benchmark.ps1

Write-Host "=== Filter Operation Latency ===" -ForegroundColor Cyan

$processCount = 1000  # Standard test load
Setup-TestProcesses -TargetCount $processCount

$process = Start-Process "rust-task-manager.exe" -PassThru
Start-Sleep -Seconds 5

# Test filter latency with various search terms
$searchTerms = @(
    @{ Term = "c"; ExpectedMatches = 150 },        # Single char (many matches)
    @{ Term = "chr"; ExpectedMatches = 50 },       # Prefix (moderate matches)
    @{ Term = "chrome"; ExpectedMatches = 10 },    # Full word (few matches)
    @{ Term = "xyz123"; ExpectedMatches = 0 },     # No matches
    @{ Term = "cpu:>50"; ExpectedMatches = 5 }     # Advanced filter
)

$results = @()

foreach ($search in $searchTerms) {
    Write-Host "`nTesting filter: '$($search.Term)'"

    # Type search term character-by-character (realistic)
    $charTimings = @()
    foreach ($char in $search.Term.ToCharArray()) {
        $startTime = Get-Date

        # Simulate keypress
        Invoke-UIAutomation -ProcessId $process.Id -Action "TypeCharacter" -Character $char

        # Wait for filter update (app should respond within 100ms per FR-005)
        $updateTime = Wait-ForBenchmarkSignal -ProcessId $process.Id -Signal "FILTER_UPDATE" -TimeoutSeconds 1
        $elapsed = ((Get-Date) - $startTime).TotalMilliseconds

        $charTimings += $elapsed
    }

    $avgLatency = ($charTimings | Measure-Object -Average).Average
    $maxLatency = ($charTimings | Measure-Object -Maximum).Maximum

    $results += [PSCustomObject]@{
        SearchTerm = $search.Term
        AvgLatency_ms = [math]::Round($avgLatency, 2)
        MaxLatency_ms = [math]::Round($maxLatency, 2)
        P95_ms = [math]::Round(($charTimings | Sort-Object)[([math]::Floor($charTimings.Count * 0.95))], 2)
    }

    # Clear filter
    Invoke-UIAutomation -ProcessId $process.Id -Action "ClearFilter"
    Start-Sleep -Milliseconds 500
}

$process | Stop-Process -Force
Cleanup-TestProcesses

# Display results
$results | Format-Table -AutoSize

# Validation: FR-005 requires <100ms real-time filter response
$maxP95 = ($results | Measure-Object P95_ms -Maximum).Maximum
if ($maxP95 -lt 100) {
    Write-Host "`n✅ PASS: Filter latency <100ms P95 (FR-005)" -ForegroundColor Green
} else {
    Write-Host "`n❌ FAIL: Filter latency exceeds 100ms" -ForegroundColor Red
}

# Ideal target: <50ms for instant feel
$avgLatency = ($results.AvgLatency_ms | Measure-Object -Average).Average
if ($avgLatency -lt 50) {
    Write-Host "✅ EXCELLENT: Average latency <50ms (instant feel)" -ForegroundColor Green
}
```

**Success Criteria**:
- ✅ Average per-character latency: <50ms (instant feel)
- ✅ P95 latency: <100ms (FR-005 requirement)
- ✅ Maximum latency: <150ms (acceptable worst case)
- ✅ No typing lag even with 2000 processes

---

### 3.4 Window Resize Performance

**Objective**: Measure smooth resize with <16ms frame time

**Test Protocol**:

```powershell
# resize-benchmark.ps1

Write-Host "=== Window Resize Performance ===" -ForegroundColor Cyan

$process = Start-Process "rust-task-manager.exe" -PassThru
Start-Sleep -Seconds 5

# Simulate window resize via automation
# Drag from 800x600 to 1920x1080 over 2 seconds
$resizeSizes = @(
    @(800, 600),
    @(1000, 700),
    @(1200, 800),
    @(1400, 900),
    @(1600, 1000),
    @(1800, 1080),
    @(1920, 1080)
)

$frameTimings = @()

foreach ($size in $resizeSizes) {
    $startTime = Get-Date

    # Resize window
    Invoke-UIAutomation -ProcessId $process.Id -Action "ResizeWindow" -Width $size[0] -Height $size[1]

    # Measure time to complete redraw
    $frameTime = Wait-ForBenchmarkSignal -ProcessId $process.Id -Signal "PAINT_COMPLETE" -TimeoutSeconds 1
    $elapsed = ((Get-Date) - $startTime).TotalMilliseconds

    $frameTimings += $elapsed
    Start-Sleep -Milliseconds 100  # Pause between resizes
}

$avgFrameTime = ($frameTimings | Measure-Object -Average).Average
$maxFrameTime = ($frameTimings | Measure-Object -Maximum).Maximum

Write-Host "`nResize Frame Timing:"
Write-Host "  Average: $([math]::Round($avgFrameTime, 2)) ms"
Write-Host "  Max:     $([math]::Round($maxFrameTime, 2)) ms"

$process | Stop-Process -Force

# Validation: <16ms per frame (60 FPS)
if ($maxFrameTime -lt 16) {
    Write-Host "✅ PASS: Resize maintains 60 FPS" -ForegroundColor Green
} elseif ($maxFrameTime -lt 33) {
    Write-Host "⚠️  ACCEPTABLE: Resize at 30+ FPS" -ForegroundColor Yellow
} else {
    Write-Host "❌ FAIL: Resize causes visible lag" -ForegroundColor Red
}
```

**Success Criteria**:
- ✅ Average frame time: <16ms (60 FPS)
- ✅ Maximum frame time: <16ms (no dropped frames)
- ✅ Consistent performance across all window sizes
- ✅ No flicker or tearing

---

### 3.5 Chart Update Performance

**Objective**: Validate 60+ FPS graph rendering (SC-005)

**Test Protocol**:

```powershell
# chart-performance-benchmark.ps1

Write-Host "=== Chart Update Performance ===" -ForegroundColor Cyan

$process = Start-Process "rust-task-manager.exe" -PassThru
Start-Sleep -Seconds 5

# Switch to Performance tab (shows live graphs)
Invoke-UIAutomation -ProcessId $process.Id -Action "SwitchTab" -Tab "Performance"
Start-Sleep -Seconds 2

# Measure frame times for 60 seconds
Write-Host "`nMeasuring graph rendering for 60 seconds (1Hz data update)..."
$frameTimes = @()
$startTime = Get-Date

while (((Get-Date) - $startTime).TotalSeconds -lt 60) {
    $frameTime = Wait-ForBenchmarkSignal -ProcessId $process.Id -Signal "FRAME_RENDERED" -TimeoutSeconds 1
    $frameTimes += $frameTime
}

# Analyze frame times
$avgFrameTime = ($frameTimes | Measure-Object -Average).Average
$fps = 1000 / $avgFrameTime
$frameDrops = ($frameTimes | Where-Object { $_ -gt 16.67 }).Count  # Frames slower than 60 FPS
$frameDropPercent = $frameDrops / $frameTimes.Count * 100

Write-Host "`nGraph Rendering Statistics:"
Write-Host "  Total Frames:    $($frameTimes.Count)"
Write-Host "  Avg Frame Time:  $([math]::Round($avgFrameTime, 2)) ms"
Write-Host "  Avg FPS:         $([math]::Round($fps, 1))"
Write-Host "  Frame Drops:     $frameDrops ($([math]::Round($frameDropPercent, 2))%)"
Write-Host "  Min Frame Time:  $([math]::Round(($frameTimes | Measure-Object -Minimum).Minimum, 2)) ms"
Write-Host "  Max Frame Time:  $([math]::Round(($frameTimes | Measure-Object -Maximum).Maximum, 2)) ms"
Write-Host "  P99 Frame Time:  $([math]::Round(($frameTimes | Sort-Object)[([math]::Floor($frameTimes.Count * 0.99))], 2)) ms"

$process | Stop-Process -Force

# Validation: SC-005 requires 60+ FPS
if ($fps -ge 60 -and $frameDropPercent -lt 5) {
    Write-Host "`n✅ PASS: Chart maintains 60+ FPS with <5% frame drops (SC-005)" -ForegroundColor Green
} else {
    Write-Host "`n❌ FAIL: Chart rendering below 60 FPS target" -ForegroundColor Red
}
```

**Success Criteria**:
- ✅ Average FPS: ≥60 (SC-005)
- ✅ Frame drops: <5% of frames
- ✅ P99 frame time: <20ms (occasional spikes acceptable)
- ✅ Consistent performance over long periods (no degradation)

---

### 3.6 Input Latency Measurement

**Objective**: Validate <16ms input-to-visual feedback (SC-007)

**Test Protocol**:

```powershell
# input-latency-benchmark.ps1

Write-Host "=== Input Latency Benchmark ===" -ForegroundColor Cyan

$process = Start-Process "rust-task-manager.exe" -PassThru
Start-Sleep -Seconds 5

# Test various input types
$inputTests = @(
    @{ Type = "MouseClick"; Action = "Click button"; Target = "RefreshButton" },
    @{ Type = "KeyPress"; Action = "Press F5"; Target = "F5" },
    @{ Type = "KeyPress"; Action = "Press Delete"; Target = "Delete" },
    @{ Type = "MouseHover"; Action = "Hover over row"; Target = "ProcessRow" },
    @{ Type = "Scroll"; Action = "Scroll list"; Target = "ProcessList" }
)

$results = @()

foreach ($test in $inputTests) {
    Write-Host "`nTesting: $($test.Action)"

    $latencies = @()
    for ($i = 0; $i -lt 50; $i++) {
        $startTime = Get-Date

        # Perform input action
        Invoke-UIAutomation -ProcessId $process.Id -Action $test.Action -Target $test.Target

        # Measure time to visual feedback (detected via screen capture or instrumentation)
        $feedbackTime = Wait-ForBenchmarkSignal -ProcessId $process.Id -Signal "VISUAL_FEEDBACK" -TimeoutSeconds 1
        $elapsed = ((Get-Date) - $startTime).TotalMilliseconds

        $latencies += $elapsed
        Start-Sleep -Milliseconds 100
    }

    $avgLatency = ($latencies | Measure-Object -Average).Average
    $p95Latency = ($latencies | Sort-Object)[([math]::Floor($latencies.Count * 0.95))]

    $results += [PSCustomObject]@{
        InputType = $test.Type
        Action = $test.Action
        AvgLatency_ms = [math]::Round($avgLatency, 2)
        P95_ms = [math]::Round($p95Latency, 2)
        MaxLatency_ms = [math]::Round(($latencies | Measure-Object -Maximum).Maximum, 2)
    }
}

$process | Stop-Process -Force

# Display results
$results | Format-Table -AutoSize

# Validation: SC-007 requires <16ms input latency
$maxP95 = ($results | Measure-Object P95_ms -Maximum).Maximum
if ($maxP95 -lt 16) {
    Write-Host "`n✅ PASS: Input latency <16ms (SC-007)" -ForegroundColor Green
} else {
    Write-Host "`n❌ FAIL: Input latency exceeds 16ms" -ForegroundColor Red
}
```

**Success Criteria**:
- ✅ All input types: <16ms average latency (SC-007)
- ✅ P95 latency: <20ms (allow for occasional spikes)
- ✅ Maximum latency: <33ms (one frame at 30 FPS)
- ✅ Consistent across all interaction types

---

## 4. FEATURE COMPARISON BENCHMARKS

### 4.1 Process Enumeration Completeness

**Objective**: Validate that all processes are detected (no missing processes vs competitors)

**Test Protocol**:

```powershell
# process-enumeration-completeness.ps1

Write-Host "=== Process Enumeration Completeness ===" -ForegroundColor Cyan

$applications = @(
    @{ Name = "Rust Task Manager"; Path = "rust-task-manager.exe" },
    @{ Name = "Windows Task Manager"; Path = "taskmgr.exe" },
    @{ Name = "Process Explorer"; Path = "C:\\Tools\\procexp64.exe" },
    @{ Name = "Process Hacker"; Path = "C:\\Tools\\ProcessHacker.exe" }
)

$results = @()

foreach ($app in $applications) {
    Write-Host "`nTesting: $($app.Name)"

    # Launch application
    $process = Start-Process $app.Path -PassThru
    Start-Sleep -Seconds 10  # Allow full enumeration

    # Export process list (via automation or screenshot OCR)
    $processList = Export-ProcessListFromApp -ProcessId $process.Id

    $processCount = $processList.Count
    $uniquePIDs = ($processList | Select-Object -Unique PID).Count

    $results += [PSCustomObject]@{
        Application = $app.Name
        ProcessCount = $processCount
        UniquePIDs = $uniquePIDs
    }

    $process | Stop-Process -Force
    Start-Sleep -Seconds 2
}

# Display comparison
$results | Format-Table -AutoSize

# Validation: Should match or exceed other tools
$ourCount = ($results | Where-Object Application -eq "Rust Task Manager").ProcessCount
$taskMgrCount = ($results | Where-Object Application -eq "Windows Task Manager").ProcessCount
$maxCount = ($results | Measure-Object ProcessCount -Maximum).Maximum

Write-Host "`nCompleteness Analysis:"
Write-Host "  Our Count:         $ourCount"
Write-Host "  Task Manager:      $taskMgrCount"
Write-Host "  Max (any tool):    $maxCount"

if ($ourCount -ge $maxCount * 0.95) {  # Within 5% of best tool
    Write-Host "✅ PASS: Process enumeration is complete" -ForegroundColor Green
} else {
    Write-Host "❌ FAIL: Missing processes compared to competitors" -ForegroundColor Red
}
```

**Success Criteria**:
- ✅ Enumerate all user processes (standard privileges)
- ✅ Enumerate system processes (admin privileges)
- ✅ Match or exceed Windows Task Manager count
- ✅ Detect processes within 100ms of creation

---

### 4.2 Metric Accuracy Validation

**Objective**: Verify CPU, memory, disk, network metrics match ground truth

**Test Protocol**:

```powershell
# metric-accuracy-benchmark.ps1

Write-Host "=== Metric Accuracy Validation ===" -ForegroundColor Cyan

# Create known workload process
$workloadScript = @"
# cpu-workload.ps1
`$start = Get-Date
while (((Get-Date) - `$start).TotalSeconds -lt 60) {
    `$x = 1
    for (`$i = 0; `$i -lt 10000000; `$i++) {
        `$x = `$x * 1.001
    }
}
"@

$workloadScript | Out-File "cpu-workload.ps1" -Encoding ASCII
$workload = Start-Process "powershell.exe" -ArgumentList "-File cpu-workload.ps1" -PassThru

Start-Sleep -Seconds 5  # Let workload stabilize

# Measure with multiple tools simultaneously
$applications = @(
    @{ Name = "Rust Task Manager"; Path = "rust-task-manager.exe" },
    @{ Name = "Windows Task Manager"; Path = "taskmgr.exe" },
    @{ Name = "Process Explorer"; Path = "C:\\Tools\\procexp64.exe" }
)

$measurements = @()

foreach ($app in $applications) {
    $monitor = Start-Process $app.Path -PassThru
    Start-Sleep -Seconds 10

    # Sample metrics for workload process (30 samples over 30 seconds)
    $samples = @()
    for ($i = 0; $i -lt 30; $i++) {
        $cpu = Get-ProcessMetric -ProcessId $workload.Id -Metric "CPU" -Source $app.Name
        $memory = Get-ProcessMetric -ProcessId $workload.Id -Metric "Memory" -Source $app.Name

        $samples += [PSCustomObject]@{
            Sample = $i
            CPU_Percent = $cpu
            Memory_MB = $memory
        }

        Start-Sleep -Seconds 1
    }

    $avgCPU = ($samples.CPU_Percent | Measure-Object -Average).Average
    $avgMemory = ($samples.Memory_MB | Measure-Object -Average).Average

    $measurements += [PSCustomObject]@{
        Application = $app.Name
        AvgCPU_Percent = [math]::Round($avgCPU, 2)
        AvgMemory_MB = [math]::Round($avgMemory, 2)
    }

    $monitor | Stop-Process -Force
}

# Ground truth from Performance Counters
$groundTruthCPU = (Get-Counter "\Process(powershell)\% Processor Time" -SampleInterval 1 -MaxSamples 30 | 
                   Select-Object -ExpandProperty CounterSamples | 
                   Measure-Object -Property CookedValue -Average).Average / [Environment]::ProcessorCount

$measurements += [PSCustomObject]@{
    Application = "Ground Truth (PDH)"
    AvgCPU_Percent = [math]::Round($groundTruthCPU, 2)
    AvgMemory_MB = "N/A"
}

$workload | Stop-Process -Force
Remove-Item "cpu-workload.ps1"

# Display comparison
$measurements | Format-Table -AutoSize

# Validation: Our measurements should be within 5% of ground truth
$ourCPU = ($measurements | Where-Object Application -eq "Rust Task Manager").AvgCPU_Percent
$errorPercent = [math]::Abs($ourCPU - $groundTruthCPU) / $groundTruthCPU * 100

Write-Host "`nAccuracy Analysis:"
Write-Host "  Ground Truth: $([math]::Round($groundTruthCPU, 2))%"
Write-Host "  Our Reading:  $ourCPU%"
Write-Host "  Error:        $([math]::Round($errorPercent, 2))%"

if ($errorPercent -lt 5) {
    Write-Host "✅ PASS: Metric accuracy within 5% of ground truth" -ForegroundColor Green
} else {
    Write-Host "❌ FAIL: Metric accuracy error exceeds 5%" -ForegroundColor Red
}
```

**Success Criteria**:
- ✅ CPU usage: Within 5% of Performance Counter (PDH) ground truth
- ✅ Memory: Within 2% of WMI query
- ✅ Disk I/O: Within 10% of ETW trace
- ✅ Network: Within 5% of GetExtendedTcpTable

---

### 4.3 Refresh Rate Consistency

**Objective**: Verify monitoring updates at configured rate without drift

**Test Protocol**:

```powershell
# refresh-rate-consistency.ps1

Write-Host "=== Refresh Rate Consistency ===" -ForegroundColor Cyan

$refreshRates = @(10, 5, 1, 0.5, 0.1)  # Hz

$results = @()

foreach ($rate in $refreshRates) {
    Write-Host "`nTesting refresh rate: $rate Hz"

    $process = Start-Process "rust-task-manager.exe" -ArgumentList "--refresh-rate $rate" -PassThru
    Start-Sleep -Seconds 5

    # Measure actual refresh intervals
    $intervals = @()
    $lastUpdate = Get-Date

    for ($i = 0; $i -lt 100; $i++) {
        Wait-ForBenchmarkSignal -ProcessId $process.Id -Signal "DATA_UPDATED" -TimeoutSeconds 30
        $now = Get-Date
        $interval = ($now - $lastUpdate).TotalMilliseconds
        $intervals += $interval
        $lastUpdate = $now
    }

    $expectedInterval = 1000 / $rate  # ms
    $actualInterval = ($intervals | Measure-Object -Average).Average
    $jitter = ($intervals | Measure-Object -StandardDeviation).StandardDeviation
    $drift = [math]::Abs($actualInterval - $expectedInterval) / $expectedInterval * 100

    $results += [PSCustomObject]@{
        ConfiguredRate_Hz = $rate
        ExpectedInterval_ms = $expectedInterval
        ActualInterval_ms = [math]::Round($actualInterval, 2)
        Jitter_ms = [math]::Round($jitter, 2)
        Drift_Percent = [math]::Round($drift, 2)
    }

    $process | Stop-Process -Force
    Start-Sleep -Seconds 2
}

# Display results
$results | Format-Table -AutoSize

# Validation: Refresh rate should be accurate within 5%
$maxDrift = ($results | Measure-Object Drift_Percent -Maximum).Maximum
if ($maxDrift -lt 5) {
    Write-Host "`n✅ PASS: Refresh rate accurate within 5%" -ForegroundColor Green
} else {
    Write-Host "`n❌ FAIL: Refresh rate drift exceeds 5%" -ForegroundColor Red
}
```

**Success Criteria**:
- ✅ Drift: <5% from configured rate
- ✅ Jitter: <10% of interval (consistent timing)
- ✅ No accumulated drift over 10 minutes
- ✅ Accurate at all supported rates (0.1Hz - 10Hz)

---

## 5. TESTING ENVIRONMENTS

### 5.1 Hardware Profile Specifications

**Low-End Profile**:
```yaml
CPU: Intel Core i3-8100 (2 cores, 4 threads, 3.6 GHz)
RAM: 4 GB DDR4 2400 MHz
Storage: SATA SSD 120 GB (SATA III, ~500 MB/s)
GPU: Intel UHD Graphics 630 (integrated)
Display: 1920x1080 @ 60 Hz
OS: Windows 10 22H2
Purpose: Minimum viable hardware validation
```

**Mid-Range Profile** (Reference System):
```yaml
CPU: Intel Core i5-10400 (6 cores, 12 threads, 4.3 GHz boost)
RAM: 16 GB DDR4 3200 MHz
Storage: NVMe SSD 512 GB (PCIe 3.0, ~3500 MB/s)
GPU: NVIDIA GTX 1660 (6 GB GDDR5)
Display: 2560x1440 @ 144 Hz
OS: Windows 11 23H2
Purpose: Typical user system, baseline for all benchmarks
```

**High-End Profile**:
```yaml
CPU: AMD Ryzen 9 5950X (16 cores, 32 threads, 4.9 GHz boost)
RAM: 64 GB DDR4 3600 MHz
Storage: NVMe SSD 2 TB (PCIe 4.0, ~7000 MB/s)
GPU: NVIDIA RTX 4080 (16 GB GDDR6X)
Display: 3840x2160 @ 144 Hz
OS: Windows 11 24H2
Purpose: Maximum performance validation, scaling tests
```

**Enterprise Server Profile**:
```yaml
CPU: Intel Xeon Platinum 8380 (40 cores, 80 threads, 3.4 GHz)
RAM: 256 GB DDR4 ECC 3200 MHz
Storage: NVMe SSD 4 TB (Enterprise, redundant)
GPU: NVIDIA Quadro P620 (workstation card)
Display: 1920x1080 @ 60 Hz (Remote Desktop typical)
OS: Windows Server 2022
Purpose: Maximum process count (2048), RDP scenarios
```

---

### 5.2 Operating System Matrix

**Test Matrix**:

| OS Version | Build Number | Architecture | Test Scenarios |
|------------|--------------|--------------|----------------|
| **Windows 10 1809** | 17763 | x64 | Minimum supported, degraded features |
| **Windows 10 21H2** | 19044 | x64 | Common stable release |
| **Windows 10 22H2** | 19045 | x64 | Final Windows 10 feature update |
| **Windows 11 21H2** | 22000 | x64 | Initial Windows 11 release |
| **Windows 11 22H2** | 22621 | x64 | Stable Windows 11 with Fluent updates |
| **Windows 11 23H2** | 22631 | x64 | Recent feature update |
| **Windows 11 24H2** | 26100 | x64 | Latest with full Fluent Design |

**Feature Validation Per OS**:
- Windows 10: Verify graceful degradation (no Mica, limited Fluent)
- Windows 11: Verify enhanced features (Mica, Acrylic, rounded corners)
- All: Verify core functionality identical

---

### 5.3 Privilege Level Testing

**Admin vs. Non-Admin Comparison**:

```powershell
# privilege-level-benchmark.ps1

Write-Host "=== Privilege Level Comparison ===" -ForegroundColor Cyan

$privilegeLevels = @("Standard", "Administrator")
$results = @()

foreach ($level in $privilegeLevels) {
    Write-Host "`nTesting as: $level"

    if ($level -eq "Administrator") {
        $process = Start-Process "rust-task-manager.exe" -Verb RunAs -PassThru
    } else {
        # Ensure running as standard user (may need to launch from standard user context)
        $process = Start-Process "rust-task-manager.exe" -PassThru
    }

    Start-Sleep -Seconds 5

    # Measure accessible features
    $processCount = Get-EnumeratedProcessCount -ProcessId $process.Id
    $canAccessSystem = Test-SystemProcessAccess -ProcessId $process.Id
    $canTerminateProtected = Test-TerminateProtected -ProcessId $process.Id

    $results += [PSCustomObject]@{
        PrivilegeLevel = $level
        ProcessCount = $processCount
        SystemProcessAccess = $canAccessSystem
        CanTerminateProtected = $canTerminateProtected
    }

    $process | Stop-Process -Force
}

$results | Format-Table -AutoSize

# Validation
$standardCount = ($results | Where-Object PrivilegeLevel -eq "Standard").ProcessCount
$adminCount = ($results | Where-Object PrivilegeLevel -eq "Administrator").ProcessCount

Write-Host "`nFeature Accessibility:"
Write-Host "  Standard User Processes: $standardCount"
Write-Host "  Admin User Processes:    $adminCount"
Write-Host "  Additional with Admin:   $($adminCount - $standardCount)"

if ($standardCount -gt 0 -and $adminCount -ge $standardCount) {
    Write-Host "✅ PASS: Application functional at both privilege levels" -ForegroundColor Green
} else {
    Write-Host "❌ FAIL: Application not functional without admin" -ForegroundColor Red
}
```

**Success Criteria**:
- ✅ Standard user: All non-privileged operations work
- ✅ Admin user: Additional access to system processes, SeDebugPrivilege
- ✅ Graceful elevation prompts when needed
- ✅ No crashes or errors in either mode

---

### 5.4 Automated Test Suite

**Master Benchmark Runner**:

```powershell
# run-all-benchmarks.ps1

param(
    [string]$OutputPath = ".\benchmark-results",
    [string]$HardwareProfile = "mid-range",
    [switch]$SkipSlowTests
)

Write-Host "=== Running Full Benchmark Suite ===" -ForegroundColor Cyan
Write-Host "Hardware Profile: $HardwareProfile"
Write-Host "Output Path: $OutputPath"

# Create output directory
New-Item -ItemType Directory -Path $OutputPath -Force | Out-Null

# System information
$systemInfo = Get-SystemInfo
$systemInfo | ConvertTo-Json | Out-File "$OutputPath\system-info.json"

# 1. Startup Performance
Write-Host "`n[1/8] Startup Performance..." -ForegroundColor Yellow
& .\cold-start-benchmark.ps1 | Tee-Object "$OutputPath\startup-cold.log"
& .\warm-start-benchmark.ps1 | Tee-Object "$OutputPath\startup-warm.log"
& .\startup-process-variance.ps1 | Tee-Object "$OutputPath\startup-variance.log"
& .\competitor-startup-benchmark.ps1 | Tee-Object "$OutputPath\startup-comparison.log"

# 2. Resource Utilization
Write-Host "`n[2/8] Resource Utilization..." -ForegroundColor Yellow
& .\memory-footprint-benchmark.ps1 | Tee-Object "$OutputPath\resource-memory.log"
& .\cpu-usage-benchmark.ps1 | Tee-Object "$OutputPath\resource-cpu.log"
& .\io-benchmark.ps1 | Tee-Object "$OutputPath\resource-io.log"

# 3. UI Responsiveness
Write-Host "`n[3/8] UI Responsiveness..." -ForegroundColor Yellow
& .\process-list-display-benchmark.ps1 | Tee-Object "$OutputPath\ui-list-display.log"
& .\sort-benchmark.ps1 | Tee-Object "$OutputPath\ui-sort.log"
& .\filter-benchmark.ps1 | Tee-Object "$OutputPath\ui-filter.log"
& .\resize-benchmark.ps1 | Tee-Object "$OutputPath\ui-resize.log"
& .\chart-performance-benchmark.ps1 | Tee-Object "$OutputPath\ui-chart.log"
& .\input-latency-benchmark.ps1 | Tee-Object "$OutputPath\ui-input-latency.log"

# 4. Feature Comparison
Write-Host "`n[4/8] Feature Comparison..." -ForegroundColor Yellow
& .\process-enumeration-completeness.ps1 | Tee-Object "$OutputPath\feature-enumeration.log"
& .\metric-accuracy-benchmark.ps1 | Tee-Object "$OutputPath\feature-accuracy.log"
& .\refresh-rate-consistency.ps1 | Tee-Object "$OutputPath\feature-refresh.log"

# 5-8. Additional tests...

# Generate summary report
Write-Host "`n[8/8] Generating Summary Report..." -ForegroundColor Yellow
$summary = Generate-BenchmarkSummary -OutputPath $OutputPath
$summary | ConvertTo-Json -Depth 5 | Out-File "$OutputPath\summary.json"
$summary | Format-List | Out-File "$OutputPath\summary.txt"

Write-Host "`n✅ Benchmark suite complete!" -ForegroundColor Green
Write-Host "Results saved to: $OutputPath"
```

---

**END OF PART 2**

**Next Steps**:
1. Implement instrumentation in Rust codebase (`--benchmark-mode` flag, named pipes, timing markers)
2. Create PowerShell automation scripts for all benchmarks
3. Set up CI/CD pipeline to run benchmarks on every PR
4. Establish performance regression detection thresholds
5. Generate HTML dashboard for benchmark visualization

**Cross-References**:
- Part 1: Startup Performance, Resource Utilization, I/O
- Spec: `../spec.md` (Success Criteria SC-001 through SC-015)
- Plan: `../plan.md` (Phase 6 optimization targets)
