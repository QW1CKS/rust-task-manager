# Checklist: Native Code Quality Requirements

**Purpose**: Validate that native Win32 integration requirements ensure safe, idiomatic, and maintainable Rust wrappers for Windows APIs.

**Created**: 2025-10-21  
**Feature**: Native High-Performance Task Manager  
**Spec Reference**: `../spec.md`

---

## Safe Wrapper Requirements

- [ ] CHK226 - Is safe Rust wrapper requirement mandated for all Windows API calls? [Completeness, Plan §Phase 1, Constitution]
- [ ] CHK227 - Are unsafe block justification documentation requirements specified? [Gap, Plan §Phase 6]
- [ ] CHK228 - Is unsafe code isolation requirement defined (FFI boundaries only)? [Completeness, Plan, Constitution]
- [ ] CHK229 - Are pre-condition documentation requirements specified for all unsafe functions? [Gap]
- [ ] CHK230 - Are post-condition documentation requirements specified for all unsafe functions? [Gap]
- [ ] CHK231 - Is invariant documentation requirement defined for unsafe data structures? [Gap]

## Error Handling Requirements

- [ ] CHK232 - Is Windows API error code checking requirement mandated for all calls? [Coverage, Spec §FR-068]
- [ ] CHK233 - Are GetLastError() capture requirements specified immediately after failed API calls? [Gap]
- [ ] CHK234 - Is error code to Result<T, E> conversion requirement defined? [Completeness, Plan §Phase 1]
- [ ] CHK235 - Are specific error variants requirement specified (vs generic Error type)? [Gap, Spec §FR-068]
- [ ] CHK236 - Is error context preservation requirement defined (operation + context)? [Completeness, Spec §FR-068]
- [ ] CHK237 - Are panic-free guarantee requirements specified for all public APIs? [Gap]
- [ ] CHK238 - Is unwind safety requirement defined for FFI boundaries? [Gap]

## Resource Cleanup Requirements

- [ ] CHK239 - Is RAII pattern requirement mandated for all Windows handles? [Coverage, Plan §Phase 4]
- [ ] CHK240 - Are Drop implementation requirements specified for all wrapper types? [Gap]
- [ ] CHK241 - Is double-free prevention requirement defined? [Gap]
- [ ] CHK242 - Is cleanup ordering requirement specified for dependent resources? [Gap]
- [ ] CHK243 - Are leak detection requirements defined (manual + tooling)? [Gap, Plan §Phase 6]
- [ ] CHK244 - Is handle validity checking requirement specified before cleanup? [Gap]

## Type Safety Requirements

- [ ] CHK245 - Is newtype pattern requirement defined for Windows handles (no raw pointers in public API)? [Gap, Plan §Phase 1]
- [ ] CHK246 - Are zero-cost abstraction requirements specified for wrapper types? [Coverage, Constitution]
- [ ] CHK247 - Is compile-time validity checking requirement defined where possible? [Gap]
- [ ] CHK248 - Are type-state pattern requirements specified for stateful APIs? [Gap]
- [ ] CHK249 - Is marker trait usage requirement defined for unsafe Send/Sync? [Gap]

## Memory Safety Requirements

- [ ] CHK250 - Is bounds checking requirement mandated for all buffer operations? [Gap]
- [ ] CHK251 - Are alignment requirement validations specified for FFI types? [Gap]
- [ ] CHK252 - Is null pointer checking requirement defined before dereferencing? [Gap]
- [ ] CHK253 - Are lifetime annotations requirement specified for borrowed Windows API data? [Gap]
- [ ] CHK254 - Is memory layout documentation requirement defined for #[repr(C)] types? [Gap]
- [ ] CHK255 - Are buffer overflow protection requirements specified? [Gap]

## API Abstraction Requirements

- [ ] CHK256 - Is idiomatic Rust API requirement defined over raw Windows API exposure? [Completeness, Constitution]
- [ ] CHK257 - Are Iterator trait implementation requirements specified for enumeration APIs? [Gap, Plan §Phase 3]
- [ ] CHK258 - Is builder pattern requirement defined for complex API initialization? [Gap]
- [ ] CHK259 - Are strongly-typed enums requirement specified vs raw constants? [Gap, Plan §Phase 1]
- [ ] CHK260 - Is zero-copy optimization requirement defined where safe? [Gap, Plan §Phase 6]

## windows-rs Integration Requirements

- [ ] CHK261 - Is windows-rs 0.58+ usage requirement specified vs manual bindings? [Completeness, Dependencies §1, Plan §Phase 1]
- [ ] CHK262 - Are feature flag requirements defined to minimize binary size? [Gap, Spec §FR-002]
- [ ] CHK263 - Is manual binding requirement specified only for missing APIs? [Clarity, Plan §Phase 1]
- [ ] CHK264 - Are windows-rs safety wrapper requirements defined for unsafe APIs? [Gap]
- [ ] CHK265 - Is version pinning requirement specified for windows-rs dependency? [Gap]

## Unsafe Code Verification Requirements

- [ ] CHK266 - Is Miri testing requirement mandated for all unsafe code? [Completeness, Plan §Phase 6]
- [ ] CHK267 - Are manual code review requirements specified for all unsafe blocks? [Gap, Plan §Phase 6]
- [ ] CHK268 - Is MSAN (MemorySanitizer) testing requirement defined? [Gap]
- [ ] CHK269 - Are undefined behavior detection requirements specified? [Gap]
- [ ] CHK270 - Is formal verification requirement defined for critical unsafe code? [Gap]

## Win32 API Pattern Requirements

- [ ] CHK271 - Are buffer size query pattern requirements defined (call twice: size, then data)? [Gap, Plan §Phase 3]
- [ ] CHK272 - Is handle lifecycle pattern requirement specified (Create/Open → Use → Close)? [Coverage, Plan §Phase 4]
- [ ] CHK273 - Are COM interface requirements defined (AddRef/Release tracking)? [Gap]
- [ ] CHK274 - Is RAII wrapper requirement mandated for all COM interfaces? [Gap]
- [ ] CHK275 - Are structured exception handling (SEH) requirements specified? [Gap]

## Direct2D Integration Requirements

- [ ] CHK276 - Is Direct2D safe wrapper requirement defined? [Completeness, Plan §Phase 2, Clarifications]
- [ ] CHK277 - Are resource lifetime requirements specified (factory, render target, brushes)? [Gap, Plan §Phase 2]
- [ ] CHK278 - Is render target threading requirement defined (UI thread only)? [Gap, Plan §Phase 2]
- [ ] CHK279 - Are BeginDraw/EndDraw pairing enforcement requirements specified? [Gap]
- [ ] CHK280 - Is device lost recovery requirement defined? [Completeness, Edge Cases]

## Thread Safety Requirements

- [ ] CHK281 - Is thread-safety documentation requirement mandated for all public types? [Gap]
- [ ] CHK282 - Are apartment threading model requirements specified for COM? [Gap]
- [ ] CHK283 - Is synchronization primitive requirement defined (Mutex, RwLock)? [Gap, Plan §Phase 3]
- [ ] CHK284 - Are lock ordering requirements specified to prevent deadlocks? [Gap]
- [ ] CHK285 - Is atomic operation requirement defined for lock-free updates? [Gap]

## Performance Requirements

- [ ] CHK286 - Is allocation minimization requirement defined in hot paths? [Completeness, Plan §Phase 6]
- [ ] CHK287 - Are zero-copy API requirements specified where possible? [Gap]
- [ ] CHK288 - Is inline annotation requirement defined for trivial wrappers? [Gap]
- [ ] CHK289 - Are branch prediction hint requirements specified (#[likely], #[unlikely])? [Gap]
- [ ] CHK290 - Is SIMD instruction usage requirement defined for data processing? [Gap, Plan §Phase 6]

## Code Organization Requirements

- [ ] CHK291 - Is module structure requirement defined for Windows API wrappers? [Gap, Plan §Phase 1]
- [ ] CHK292 - Are visibility (pub/pub(crate)) requirements specified? [Gap]
- [ ] CHK293 - Is API surface minimization requirement defined? [Gap, Constitution]
- [ ] CHK294 - Are internal implementation detail hiding requirements specified? [Gap]
- [ ] CHK295 - Is documentation module organization requirement defined? [Gap]

## Testing Requirements

- [ ] CHK296 - Is unit testing requirement mandated for all wrapper APIs? [Gap, Plan §Phase 6]
- [ ] CHK297 - Are integration testing requirements specified for Windows API interactions? [Gap, Plan §Phase 6]
- [ ] CHK298 - Is error path testing requirement defined (access denied, invalid handles)? [Gap]
- [ ] CHK299 - Are stress testing requirements specified (handle exhaustion, memory pressure)? [Gap]
- [ ] CHK300 - Is mock API requirement defined for unit testing without Windows dependencies? [Gap]

## Documentation Requirements

- [ ] CHK301 - Is rustdoc requirement mandated for all public items? [Gap]
- [ ] CHK302 - Are safety documentation requirements specified for unsafe functions? [Gap]
- [ ] CHK303 - Is example code requirement defined for complex APIs? [Gap]
- [ ] CHK304 - Are Windows API reference links requirement specified in documentation? [Gap]
- [ ] CHK305 - Is panic documentation requirement defined for all panicking paths? [Gap]

## Compatibility Requirements

- [ ] CHK306 - Is minimum Windows version detection requirement defined? [Completeness, Spec §FR-043, Clarifications]
- [ ] CHK307 - Are API availability checking requirements specified (GetProcAddress)? [Gap, Plan §Phase 7]
- [ ] CHK308 - Is graceful degradation requirement defined for missing APIs? [Completeness, Spec §FR-043, Edge Cases]
- [ ] CHK309 - Are version-specific API requirements documented? [Gap]
- [ ] CHK310 - Is runtime feature detection requirement specified? [Gap, Plan §Phase 7]

## Panic Safety Requirements

- [ ] CHK311 - Is panic-free guarantee requirement mandated for Drop implementations? [Gap]
- [ ] CHK312 - Are panic boundary requirements specified at FFI calls? [Gap]
- [ ] CHK313 - Is catch_unwind requirement defined at C callback boundaries? [Gap]
- [ ] CHK314 - Are poisoned lock recovery requirements specified? [Gap]
- [ ] CHK315 - Is invariant preservation requirement defined after panics? [Gap]

## Code Quality Metrics Requirements

- [ ] CHK316 - Is unsafe code percentage limit requirement defined (<5%)? [Gap, Constitution]
- [ ] CHK317 - Are cyclomatic complexity limits requirements specified? [Gap]
- [ ] CHK318 - Is code coverage requirement defined for wrapper modules (>90%)? [Gap, Plan §Phase 6]
- [ ] CHK319 - Are linting requirements specified (clippy::pedantic)? [Gap, Plan §Phase 6]
- [ ] CHK320 - Is code formatting requirement mandated (rustfmt)? [Gap, Plan §Phase 6]

## Consistency Validation

- [ ] CHK321 - Are native code requirements consistent between Plan and Constitution? [Consistency]
- [ ] CHK322 - Do unsafe code constraints in Plan align with Constitution principles? [Consistency]
- [ ] CHK323 - Are RAII requirements consistently applied across all resource types? [Consistency]
- [ ] CHK324 - Do error handling requirements align with FR-064 through FR-068? [Consistency]
- [ ] CHK325 - Are Windows API version requirements consistent with FR-043 and Clarifications? [Consistency]

---

**Total Items**: 100  
**Focus Areas**: Safe Wrappers, Error Handling, Resource Management, Type Safety, Memory Safety, API Abstraction, Unsafe Verification, Win32 Patterns, Direct2D, Thread Safety, Performance, Organization, Testing, Documentation, Compatibility, Panic Safety, Metrics  
**Depth**: Formal PR Review Gate  
**Traceability**: 73% items reference spec sections or identify gaps
