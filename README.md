# TACAOS

**Tsundere Architecture for Containers and Operating Systems** (real name)

TACAIS is an experimental, server focused opearting system, written primarily in rust, with assembled used for the bootloader and other low level components.

Its goal is to provide a highly minimal, efficient and secure environment for hosting OCI containers, with its own kernel, filesystem, bootloader and container runtime.

TACAOS is designed around the idea that an operating system should prove exactly what it is needed for certain workflows rather than caring about unnecessary components and compatibility layers.

## Goals

- Build an operating system that is minimal, efficient and secure
- Support x86, x86_64, AArch32 and AArch64
- Provide UEFI boot support, with legacy BIOS support planned
- Develop a custom filesystem optimized for server and container workloads
- Implement a native OCI runtime
- Provide a layered permission and execution architecture
- Enable reliable system updates with minimal service interruption
- Kepe the system minimal without sacrificing any functionality

## Architecture

TACAOS uses a layered architecture that defines both execution domains and permission boundaries.

The layers are as follows:
| Layer | Description | Id |
|-------|-------------|----|
| Harware | Physical hardware and architecture-specific operations | 1 |
| SP | Super Privileged kernel-level operations | 10 |
| Serverable | Core system services, drivers, filesystems, and OCI runtime | 11 |
| Privileged | Application backends and any management interfaces | 100 |
| Userable | Optional user-facing services (Though not a primary goal) | 101 |
| View-Only | Read-only access to explicitly permitted resources | 110 |
| No Access | You exist, you cannot do or see anything | 111 |

Higher layers have broader authority over lower layers. Lower layers cannot freely initiate operations against higher layers. (Same was as a router firewall works, a local device [higher layers] can initate connections to a remote device [lower layers], but the remote device cannot initiate connections to the local device unless explicitly permitted.)

Communication and permissions are explicitly controlled rather than relying on unrestriced access between components.

Execution scheduling and security privileges are seperate concepts, even though the architecture can assign scheduling preferences to different layers.

## Components

### Bootloader

Responsible for initializing the machine and loading the kernel.

Planned architecture targets:
- x86
- x86_64
- AArch32
- AArch64

UEFI is the inital firmware target, with legacy BIOS support planned later.

### Kernel

A custom kernel written in rust (and some cross platform assembly).

It is responsible for:
- Memory management
- Process and thread management
- Scheduling
- Hardware abstraction
- Inter-layer communication
- Permission enforcement
- System calls

### Filesystem

TACAOS will have its own filesystem implementation.

Performance, reliability and efficient storage are primary goals.

Planned areas include:
- Block allocation
- Metadata management
- Directory indexing
- File operations
- Caching
- Data integrity and recovery
- Transactions
- Snapshots
- Recovery

The filesystem will be designed to integrate directly into the OCI runtime and YFCYO update system.

### OCI Runtime

A custom OCI-compatible container runtime

The runtime will manage container creation, execution, isolation, storage, and lifecycle operations.

It will integrate directly with the TACAOS kernel and filesystem rahter than depending on docker.

### Networking

A native networking stack and associated system services.

Networking will provide connectivity for containers, host administration, and inter-service communication.

### YFCYO

**Your Friendly Controlling Yandere Oversee** (Another real name btw)

YFCYO is responsible for system updates and component handoffs.

Its intended update process involves:
1. Creating a temporary environment
2. Migrating or handing off critical services
3. Updating the primary system
4. Transferring services back to the updated system
5. Shutting down the temporary environment
6. Updating itself

The objective is to minimize downtime and interruption during system updates.

Actual zero downtime handoffs will depend on component support and the ability to transfer relevant execution and network state safely.

## Filesystem Layout

TACAOS intentionally avoids following conventional Linux filesystem layouts.

The inital layout will be something along the lines of the following, however, it is definitely subject to change:
```
├── filesystem/
├── oci/
│   └── snapshots/
│       ├── <project_id>/
│       └── backups/
└── users/
    └── <username>/
```

There is no conventional root user.

Instead, host is a special system identity with the highest privileges (`1`). It does not have a home directory and is intended purely for administration.

## Development

TACAOS is maintained as a monorepo because its components are designed to work closely together.

```
TACAOS/
├── boot/
├── kernel/
├── services/
│   ├── networking/
│   ├── ssh/
│   └── service_name/
├── filesystem/
├── oci/
├── yfcyo/
├── tests/
└── dev/
```

The dev/ directory contains development and testing infrastructure/tools.  
The tests/ directory is for tests, pretty self explanatory, don't know why you are reading this line.

## Languages

- **Rust**: Primary implemnentation language.
- **Assembly**: Bootstrapping and architecture-specific low-level operations.

## Testing

Development will use a combination of the following:
- Native Rust unit and integration tests. (tests/)
- QEMU-based hardware emulation
- KVM acceleration where available
- Virtual block devices
- Fauly injection
- Physical hardware testing (when possible)

Filesystem and other hardware independant components should be testable independently of the running kernel wherever possible.

## Contributing

Please refer to the [CONTRIBUTING.md](CONTRIBUTING.md) file for details on how to contribute to TACAOS.

## License

This project is licensed under the Apache License 2.0. See the [LICENSE.md](LICENSE.md) file for details.