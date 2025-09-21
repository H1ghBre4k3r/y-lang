# Constants

Constants in Y are immutable values that are known at compile time. They provide a way to define unchanging values that can be used throughout your program, offering both clarity and potential performance benefits.

## Constant Declaration

Constants are declared using the `const` keyword:

```why
const CONSTANT_NAME: Type = value;
```

The value must be a compile-time constant expression.

## Basic Constants

### Numeric Constants

```why
const PI: f64 = 3.1415;
const MAX_SIZE: i64 = 1000;
const DEFAULT_PORT: u32 = 8080;
const GOLDEN_RATIO: f64 = 1.618;
```

### Boolean Constants

```why
const DEBUG_MODE: bool = true;
const PRODUCTION_BUILD: bool = false;
const ENABLE_LOGGING: bool = true;
```

### String Constants

```why
const APPLICATION_NAME: str = "Y Lang Compiler";
const VERSION: str = "1.0.0";
const DEFAULT_CONFIG_FILE: str = "config.yml";
```

### Character Constants

```why
const SEPARATOR: char = ',';
const NEWLINE: char = '\n';
const TAB: char = '\t';
```

## Using Constants

Constants can be used anywhere their type is expected:

```why
const MAX_USERS: i64 = 100;
const TIMEOUT_SECONDS: f64 = 30.0;

fn create_user_pool(): UserPool {
    UserPool {
        capacity: MAX_USERS,
        timeout: TIMEOUT_SECONDS
    }
}

fn is_valid_user_count(count: i64): bool {
    count <= MAX_USERS
}
```

## Examples from Y Code

### Mathematical Constants

```why
const PI: f64 = 3.1415;

fn calculate_circle_area(radius: f64): f64 {
    PI * radius * radius
}

fn calculate_circumference(radius: f64): f64 {
    2.0 * PI * radius
}
```

### Configuration Constants

```why
const BUFFER_SIZE: i64 = 1024;
const MAX_CONNECTIONS: i64 = 100;
const DEFAULT_TIMEOUT: f64 = 5.0;

fn create_server(): Server {
    Server {
        buffer_size: BUFFER_SIZE,
        max_connections: MAX_CONNECTIONS,
        timeout: DEFAULT_TIMEOUT
    }
}
```

## Global Constants

Constants can be declared at the top level and used throughout your program:

```why
const PROGRAM_VERSION: str = "2.1.0";
const MAX_RETRIES: i64 = 3;
const ERROR_THRESHOLD: f64 = 0.01;

fn main(): void {
    printf("Starting program version: ");
    printf(PROGRAM_VERSION);
    printf("\n");

    let config = Config {
        retries: MAX_RETRIES,
        threshold: ERROR_THRESHOLD
    };
}
```

## Constants vs Variables

### Constants

```why
const FIXED_VALUE: i64 = 42;  // Cannot be changed
// FIXED_VALUE = 100;         // Error: cannot modify constant
```

### Variables

```why
let mut changeable_value: i64 = 42;  // Can be changed
changeable_value = 100;              // Valid: variable is mutable
```

## Practical Examples

### System Configuration

```why
const SYSTEM_NAME: str = "Y Lang Runtime";
const VERSION_MAJOR: i64 = 1;
const VERSION_MINOR: i64 = 0;
const VERSION_PATCH: i64 = 0;

struct SystemInfo {
    name: str;
    version: str;
}

fn get_system_info(): SystemInfo {
    SystemInfo {
        name: SYSTEM_NAME,
        version: format_version(VERSION_MAJOR, VERSION_MINOR, VERSION_PATCH)
    }
}
```

### Mathematical Operations

```why
const E: f64 = 2.71828;           // Euler's number
const SQRT_2: f64 = 1.41421;      // Square root of 2
const GOLDEN_RATIO: f64 = 1.61803; // Golden ratio

fn exponential_growth(initial: f64, time: f64): f64 {
    initial * pow(E, time)
}

fn diagonal_length(side: f64): f64 {
    side * SQRT_2
}
```

### Game Development

```why
const SCREEN_WIDTH: i64 = 1920;
const SCREEN_HEIGHT: i64 = 1080;
const FPS: i64 = 60;
const GRAVITY: f64 = 9.81;

struct GameSettings {
    width: i64;
    height: i64;
    target_fps: i64;
    physics_gravity: f64;
}

fn create_game_settings(): GameSettings {
    GameSettings {
        width: SCREEN_WIDTH,
        height: SCREEN_HEIGHT,
        target_fps: FPS,
        physics_gravity: GRAVITY
    }
}
```

### Network Configuration

```why
const DEFAULT_HTTP_PORT: i64 = 80;
const DEFAULT_HTTPS_PORT: i64 = 443;
const MAX_PACKET_SIZE: i64 = 1500;
const CONNECTION_TIMEOUT: f64 = 10.0;

struct NetworkConfig {
    http_port: i64;
    https_port: i64;
    packet_size: i64;
    timeout: f64;
}

fn create_network_config(): NetworkConfig {
    NetworkConfig {
        http_port: DEFAULT_HTTP_PORT,
        https_port: DEFAULT_HTTPS_PORT,
        packet_size: MAX_PACKET_SIZE,
        timeout: CONNECTION_TIMEOUT
    }
}
```

## Constants in Expressions

Constants can be used in any expression where their type is appropriate:

```why
const BASE_SCORE: i64 = 100;
const MULTIPLIER: f64 = 1.5;

fn calculate_final_score(bonus: i64, time_factor: f64): f64 {
    (BASE_SCORE + bonus) * MULTIPLIER * time_factor
}

fn is_high_score(score: i64): bool {
    score > BASE_SCORE * 10
}
```

## Constant Arrays (Conceptual)

While Y's current syntax may not support constant arrays directly, the concept would be:

```why
// Conceptual - may not be currently supported
const PRIME_NUMBERS: &[i64] = &[2, 3, 5, 7, 11, 13, 17, 19];
const FIBONACCI: &[i64] = &[1, 1, 2, 3, 5, 8, 13, 21];
```

## Best Practices

### Naming Conventions

Use UPPER_CASE for constants to distinguish them from variables:

```why
// Good: Clear constant naming
const MAX_BUFFER_SIZE: i64 = 4096;
const DEFAULT_CHARSET: str = "UTF-8";
const ENABLE_DEBUG: bool = false;

// Less ideal: Unclear naming
const maxBufferSize: i64 = 4096;  // Looks like a variable
const size: i64 = 4096;           // Too generic
```

### Grouping Related Constants

```why
// Database configuration
const DB_HOST: str = "localhost";
const DB_PORT: i64 = 5432;
const DB_NAME: str = "ylang_db";
const DB_TIMEOUT: f64 = 30.0;

// Graphics configuration
const WINDOW_WIDTH: i64 = 1024;
const WINDOW_HEIGHT: i64 = 768;
const REFRESH_RATE: i64 = 60;
const VSYNC_ENABLED: bool = true;
```

### Documentation

Document the purpose and units of constants:

```why
// Network timeout in seconds
const NETWORK_TIMEOUT: f64 = 30.0;

// Maximum file size in bytes (10 MB)
const MAX_FILE_SIZE: i64 = 10485760;

// Frame rate in frames per second
const TARGET_FPS: i64 = 60;

// Speed of light in meters per second
const SPEED_OF_LIGHT: f64 = 299792458.0;
```

### Avoid Magic Numbers

Replace magic numbers with named constants:

```why
// Bad: Magic numbers
fn process_data(data: &[i64]): bool {
    data.length() <= 1000 && data[0] > 42
}

// Good: Named constants
const MAX_DATA_SIZE: i64 = 1000;
const MIN_THRESHOLD: i64 = 42;

fn process_data(data: &[i64]): bool {
    data.length() <= MAX_DATA_SIZE && data[0] > MIN_THRESHOLD
}
```