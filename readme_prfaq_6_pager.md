# PRFAQ: OOBE SDK

## Press Release

### Amazon Launches OOBE SDK to Transform Device Setup Experiences

SEATTLE – Amazon today announced a groundbreaking new architecture for out-of-box experiences (OOBE) across its device portfolio. The new plugin-based architecture fundamentally reimagines how setup experiences are built, deployed, and maintained, addressing long-standing challenges in the consumer electronics industry.

"This represents a paradigm shift in how we deliver first impressions to our customers," said [EXECUTIVE NAME], VP of Device Experience at Amazon. "By transforming our OOBE from static firmware to dynamic plugins, we're creating setup experiences that are consistent, updatable, and continuously improving—even for devices already in customer homes."

The new architecture dramatically improves development velocity, enabling teams to deliver experience improvements in days rather than months. Early trials show setup completion rates improved by 15%, while support contacts related to device setup have decreased by over 20%.

"Customer expectations for smart, adaptive experiences continue to rise," added [EXECUTIVE NAME]. "Our plugin-based architecture closes the gap between these expectations and our ability to meet them, while simultaneously reducing operational costs and creating new opportunities for post-setup engagement."

The architecture will begin rolling out to select devices in Q4, 2025, with portfolio-wide implementation expected by mid-next year.

## Frequently Asked Questions

### What challenges do we face with our current OOBE architecture?

Our current architecture creates significant challenges across multiple dimensions:

**Each device has its own OOBE software:** Currently, we maintain hundreds of different OOBE paths across our device portfolio. Each product line independently develops its setup flow, resulting in duplicated effort and inconsistent experiences. [Support this statement with data, for example: total number of distinct OOBE codepaths across product lines, developer hours spent reimplementing similar functionality, percentage of code duplication between product lines]

**Multiple operating systems require support:** Supporting multiple platforms (FOS, Vega, AOSP) currently forces teams to maintain parallel implementations of the same functionality. [Support this statement with data, for example: number of platforms supported, lines of code dedicated to platform-specific logic, engineering hours spent maintaining separate implementations]

**Product variations create complexity:** Even within product lines, we must account for headless devices, multi-modal interactions, and entry-level vs. premium experiences. The current approach bundles all possible paths into monolithic packages, creating bloated deployments. [Support this statement with data, for example: current deployment package sizes, percentage of unused code in typical deployments, loading time differences between monolithic vs. modular approaches]

**Duplicated common tasks waste resources:** Each team currently rebuilds fundamental tasks like device login, account linking, device registration, file copying, system feature enablement, and service restarts. [Support this statement with data, for example: number of teams implementing their own versions of these common components, defect rates in common components vs. specialized functionality, engineering hours spent maintaining redundant implementations]

### Why can't we update OOBE experiences after products ship?

**OOBE remains frozen after shipping:** Traditional OOBE workflows are hardcoded into firmware or bundled in system images, making them impossible to update without full firmware releases. Our architecture decouples OOBE from firmware through dynamically loadable plugins, allowing continuous improvement even for devices already in customer homes or warehouses. [Support this statement with data, for example: average time between OOBE improvements, percentage of devices in homes running outdated OOBE versions, customer impact metrics for devices unable to receive OOBE updates]

### How do current OOBE approaches affect customer experience?

Current OOBE approaches create several customer-facing challenges:

**Upsell during setup creates friction:** Current monetization approaches force upsell attempts during initial setup when customers are focused on getting devices operational. This creates friction and increases abandonment rates. [Support this statement with data, for example: setup abandonment rates at upsell screens, time spent on upsell screens vs. functional setup screens, conversion rates during setup vs. post-setup engagement]

**Inconsistent experiences damage brand perception:** Users currently encounter different visual languages and interaction patterns across our product portfolio, creating confusion and undermining brand cohesion. [Support this statement with data, for example: customer confusion metrics across multi-device households, support contacts related to inconsistent experiences, brand perception metrics across product lines]

### What prevents effective measurement and optimization of setup flows?

**Fragmented metrics limit optimization:** Each product line currently implements its own instrumentation, making cross-portfolio analysis nearly impossible. [Support this statement with data, for example: number of incompatible telemetry systems in use, percentage of important user journeys that cannot be tracked across products, engineering hours spent implementing redundant analytics]

**A/B testing and feature variation complexity:** The current architecture makes experimentation and feature management prohibitively complex due to feature variations, OS differences, device capabilities (headless/multimodal), and geographic restrictions (such as codec availability in certain regions). [Support this statement with data, for example: number of test configurations currently required to validate a feature across the product portfolio, time spent ensuring regional compliance, incidents of non-compliant features reaching production]

### What is the OOBE SDK and how does it unify development?

Our OOBE SDK provides a unified architecture that addresses the fragmentation problems in our current approach:

**Unifying device-specific OOBE software:** Our SDK replaces hundreds of product-specific implementations with a single unified runtime engine and modular plugin framework. This shared architecture allows all products to leverage common components while still enabling customization for specific needs. Rather than building setup flows from scratch, teams compose them from pre-built, well-tested modules, reducing development effort by up to 60% while ensuring consistent quality. This dramatically reduces the redundant development currently happening across product lines.

**Supporting multiple operating systems:** Through our platform-agnostic binary interface with cross-language ABI compatibility, we enable identical business logic to run across all platforms without recompilation or platform-specific code paths. This eliminates the need for teams to maintain parallel implementations of the same functionality for different operating systems. The interface provides guaranteed compatibility across FOS, Vega, and AOSP through a well-defined type system and shadow-versioning techniques.

### How does the architecture work?

At its core, our architecture uses a "plugins and engine" approach similar to how web browsers work with extensions. When a device starts setup, a small core engine initializes and connects to the network. Once connected, it checks a cloud service for the latest "execution plan" – essentially a recipe that describes which setup components (plugins) are needed for this specific device and in what order they should run.

The engine then dynamically downloads just the plugins needed for this device's particular setup flow. Each plugin is a self-contained module responsible for one specific task – like WiFi connection, account login, or device registration. As the customer completes each step, the engine seamlessly transitions to the next plugin in the sequence, creating a fluid experience despite the modular architecture underneath.

What makes this approach powerful is that plugins communicate through standardized events rather than direct dependencies. When the WiFi plugin successfully connects to a network, it simply announces "WiFi setup complete" – and the engine knows which plugin to load next. This event-driven approach means plugins can be updated, reordered or replaced without breaking the entire flow. Better yet, specialized teams across the company own the plugins in their domain of expertise, ensuring each component represents the best implementation of that particular feature.

The architecture extends beyond the initial setup through our Second Chance OOBE (SCOOBE) framework, which remains active after setup is complete. This allows for intelligently timed interactions after the customer has been using their device – like introducing advanced features when they're most relevant, or presenting subscription opportunities during detected idle periods – all without interrupting the critical first-use experience.

### How does the SDK handle product variation and common tasks?

Our SDK solves two key development challenges through its architecture:

**Solving product variation complexity:** Each product variation receives its tailored execution plan from the cloud at runtime, eliminating the need to bundle all possible paths into the firmware. This enables precise customization for headless devices, multi-modal interactions, and entry-level vs. premium experiences without bloating package sizes with unused code. Teams define their configuration profiles and the system dynamically delivers only what each device needs.

**Eliminating duplicated common tasks:** By centralizing core components under domain expert ownership, we eliminate the redundant implementation of critical functionality across product lines. For example, WiFi configuration code currently exists in over X different codebases, each with its own bugs and edge-case handling. Our SDK provides a single, thoroughly tested implementation maintained by networking specialists that all products can leverage, dramatically reducing development time while improving reliability. Similar efficiencies apply to:

- **Device identity and authentication** - Built and maintained by domain experts in the eCommerce Foundation Identity team who understand security protocols, token management, and account verification at a depth that product teams cannot match
- **Account linking and provisioning** - Developed once by the Device Management team who already maintains our device fleet infrastructure but currently has no control over the onboarding flows that populate it
- **Network setup and zero-touch onboarding** - Owned by the Frustration-Free Setup team with their sophisticated WiFi configuration logic that's currently reimplemented—often with critical bugs—by each product team
- **System-level operations** - Standardized under the Device Software Services team, preventing dangerous implementations for tasks like file operations and service management
- **Legal compliance screens** - Governed by the Legal team rather than being hardcoded by individual product groups, eliminating compliance risk from inconsistent implementations

### What is your Minimal Loveable Product (MLP)?

Our MLP focuses on delivering a solid foundation for plugin-based OOBE architecture that enables early adopters to realize immediate value while setting the stage for future expansion:

**Core Workflow Engine with Dynamic Plugin Architecture** - A lightweight, high-performance runtime engine written in native language (rust) with minimal memory footprint for resource-constrained devices. The platform-agnostic binary interface ensures cross-platform compatibility across FOS, Vega, AOSP etc. with consistent plugin behavior regardless of the underlying operating system.

**Execution Plan System with Remote Update Capability** - A declarative execution plan format that defines which plugins to load and their execution sequence. The engine checks for updated plans upon network connectivity and can dynamically download the latest version without firmware updates. A fallback mechanism ensures setup can continue even if download attempts fail.

**Event-Driven Plugin Communication Framework** - A disconnected intra-process communication system that enables plugins to publish events and subscribe to the events and receive payloads, creating a loosely coupled architecture. This allows plugins to be developed independently while still coordinating effectively during the setup flow.

**Advanced Security and Authentication** - JWT authentication with message-level encryption to secure all communication between components that cross memory boundaries. This system includes token validation on every request, automatic token refresh security model limiting plugin permissions.
Comprehensive Instrumentation and Logging - Built-in telemetry framework with automatic instrumentation of core OOBE metrics. Plugins can leverage aspect-oriented logging macros that add minimal overhead while providing thorough diagnostics data for debugging and optimization.

**Plugin Development Helpers** - Command-line tools to generate plugin scaffolding, templates, and boilerplate code. These accelerate development while ensuring consistency and best practices across the plugin ecosystem.

**Plugin Repository and Catalog System** - A central repository where plugins can be versioned, stored and discovered by product teams. The catalog includes metadata on ownership, dependencies, and compatibility, making it easy for teams to find existing components rather than building redundant implementations.

**Reference Implementation with Core Plugins** - A set of essential plugins covering common OOBE tasks such as Wi-Fi setup, account linking, login, and device provisioning. These serve both as functional components and implementation examples for developers building custom plugins.

**Detailed Documentation and Samples** - Comprehensive technical documentation including architecture diagrams, API references, and tutorials. Sample implementations demonstrate best practices and integration patterns for various use cases.

**Testing Environment for Plugin Development** - A sandbox environment where developers can test their plugins in isolation or as part of more complex workflows without requiring access to complete product builds.

This foundation provides immediate value through reduced development effort, standardized patterns, and enhanced flexibility, while positioning us for future enhancements like the SCOOBE framework, machine learning-driven optimizations, and integration with companion mobile applications.


### How does the SDK enable updatable OOBE experiences?

**Enabling updatable OOBE through dynamic execution plans:** Our dynamic execution plan system completely transforms how setup experiences evolve after shipping. OOBE launches with a default plan and immediately after network connectivity is established, checks for an updated execution plan from the cloud that's specific to the product, version, region, device type, and user segment. It then downloads any necessary plugins and executes the latest flow, ensuring even devices manufactured months ago receive the most current experience. This solves the "frozen OOBE" problem without requiring firmware updates.

### What is SCOOBE and how does it enhance customer engagement?

**Creating a platform for post-setup engagement:** Our SCOOBE (Second Chance OOBE) framework allows for intelligent scheduling of promotional interactions after the device is fully operational, addressing the upsell-during-setup problem. By providing a mechanism to engage users at appropriate moments after they're already enjoying their device, we can improve conversion rates while reducing setup abandonment. SCOOBE extends far beyond simple upsell opportunities - it enables numerous post-setup optimizations:

- **Delayed system maintenance** - Postpone resource-intensive operations like Over-the-Air updates until periods of detected inactivity
- **Contextual feature education** - Introduce advanced features only after detecting meaningful usage patterns
- **Intelligent data collection** - Schedule periodic customer feedback surveys or usage analytics after specific milestones (e.g., X days of ownership)
- **Opportunistic engagement** - Present relevant content or subscription offers during detected idle periods
- **Progressive device integration** - Stage the introduction of ecosystem connections to other Amazon devices based on usage patterns
- **Seasonal feature highlights** - Time the introduction of holiday-specific features or content based on calendar events

The SCOOBE framework turns device setup from a one-time event into an evolving relationship, allowing us to thoughtfully sequence interactions that might otherwise overwhelm users during initial configuration. However, successfully leveraging this capability requires cross-team alignment on timing and strategy for these extended interactions.

### How does the SDK improve the customer experience?

**Creating consistent user experiences:** The SDK creates a framework for cohesive customer journeys without dictating exact visual designs. Rather than forcing rigid screen templates, we provide common interaction models, navigation patterns, and functional flows that can be styled according to each product's needs. This preserves brand distinctiveness while ensuring that fundamental user actions like connecting to WiFi, signing in, or accepting terms work consistently across the portfolio. The result is reduced customer confusion, improved cross-device ownership, and measurably faster setup completion as customers apply mental models from one device to another.

### How does the SDK improve measurement and experimentation?

The SDK brings two key capabilities to improve measurement and testing:

**Standardizing metrics collection:** Our built-in instrumentation library automatically sends console logs, errors, information, crashes, and user interactions to a centralized cloud-based metrics system. We also provide no-code ready-to-use function attributes that plugin developers can apply to automatically collect predefined metrics without writing any instrumentation code. This creates unprecedented visibility into user journeys and abandonment patterns across the entire portfolio.

**Enabling seamless A/B testing:** Our dynamic execution plan system fundamentally solves the A/B testing challenge. By designing the SDK to fetch and execute different plans based on device type, region, product version, geographic location and other factors, we enable seamless experimentation without firmware changes. Product teams can define multiple execution plans that vary specific components or flows, then remotely control which customers receive each variant through our cloud distribution system. This allows for rapid experimentation and data-driven optimization previously impossible with firmware-embedded experiences.

### Why is your team best positioned to build this SDK?

We believe our team brings together a unique combination of skills and perspectives needed for this challenge. Our experience spans across all major Operating Systems (FOS, Vega, and AOSP), providing us with insights into the technical nuances of each system. Having collectively worked on Frustration Free Setup and a few OOBE implementations for certain Amazon devices, we've gained first-hand understanding of the pain points both developers and customers face during setup.
Our engineering team includes specialists in resource-constrained environments who understand how to build lightweight, efficient systems that perform well even on entry-level devices. We've also had the opportunity to develop several instrumentation frameworks currently used within Amazon, which has deepened our appreciation for consistent, meaningful analytics.
Perhaps most importantly, our position as a horizontal platform team has allowed us to build relationships across organizational boundaries. We've established working partnerships with most product lines and platform teams across amazon. These connections have helped us understand the challenges from multiple perspectives and identify common patterns that might not be visible from within a single product team.
While we don't claim to have all the answers, we believe our cross-cutting visibility puts us in a good position to create a solution that addresses the needs of all stakeholders. As a team chartered with building horizontal platform capabilities, we're committed to creating tools that empower our partner teams to deliver exceptional customer experiences.

### Who would be very happy about this SDK?

Several key stakeholders would enthusiastically welcome this solution:

- **Product Managers** - Will gain the ability to quickly iterate on setup flows without firmware dependencies, reducing time-to-market for improvements
- **UX Designers** - Can create more consistent experiences across devices while still maintaining product distinctiveness
- **Support Teams** - Will see reduced setup-related contacts through improved diagnostics and fewer customer errors
- **Analytics Teams** - Will finally have comparable metrics across product lines for benchmarking and optimization
- **Legal/Privacy Teams** - Can directly update compliance language across all devices when regulations change
- **Domain Expert Teams** - Can implement their expertise once and see it properly deployed across the portfolio

Additionally, customers who own multiple Amazon devices will benefit from more consistent, intuitive setup experiences that build on their existing knowledge rather than requiring them to learn new patterns for each device.

### Who would not be very happy about this SDK?

Some groups may have initial reservations:

- **Platform Engineering Teams** - May resist architectural changes that require modifications to their core systems
- **Product Teams with Unique Requirements** - May worry about losing the flexibility to create fully customized experiences
- **Teams with Existing OOBE Investments** - Could be concerned about sunk costs in their current solutions
- **Security Teams** - May initially be concerned about the dynamic download and execution model

We've proactively addressed many of these concerns in our design. For example, the SDK provides extensive customization points while still maintaining core consistency, and our security model includes rigorous signing, verification, and encryption protocols.

### What happens if we don't do anything and maintain the status quo?

Continuing with our current fragmented approach would have serious implications for both our business and customer experience:

**Competitive disadvantage:** While our competitors increasingly deliver seamless, updatable setup experiences, our static OOBE implementations will feel increasingly dated. This gap will widen over time as customer expectations continue to rise, potentially affecting purchase decisions and brand loyalty.

**Unsustainable engineering costs:** As our device portfolio expands, the cost of maintaining hundreds of separate OOBE implementations will grow exponentially. Without standardization, we'll continue spending millions annually on redundant development work across product lines rather than innovative features that differentiate our products.

**Persistent quality problems:** The inconsistent implementation of core functionality will perpetuate known issues in setup flows. Without domain expert ownership, critical components like WiFi setup and account linking will continue to have varying defect rates across products, leading to avoidable customer frustration and support contacts.

**Inability to respond to market changes:** When regulatory requirements change or new competitive features emerge, our current model requires separate updates to numerous codebases—often through full firmware releases. This makes us dangerously slow to respond in a rapidly evolving market.

**Lost data opportunities:** Without standardized instrumentation across the portfolio, we'll continue making product decisions based on incomplete or inconsistent data. The inability to perform portfolio-wide analysis means we can't identify and address systemic issues affecting customer satisfaction.

Current alternatives either continue this fragmented per-product approach or offer limited middleware solutions that only address specific parts of the problem. By contrast, our architecture is the first comprehensive system designed to standardize OOBE across Amazon's entire device portfolio while simultaneously improving development efficiency, customer experience, and post-setup engagement.

### What roadblocks could you anticipate in adopting this SDK?

We anticipate several implementation challenges as we move forward with our project. One of the primary concerns is the complexity of platform integration. Ensuring that the core engine works consistently across all operating systems is a significant task. Additionally, we need to determine a strategy for backward compatibility to support legacy devices already in production.
Another challenge is the transfer of knowledge to product teams. Educating these teams on the new development paradigm is crucial for successful implementation. Ownership transitions also pose a challenge, as we need to move components from product teams to domain expert teams. This shift requires careful planning and coordination.
Change management is another critical aspect. Shifting organizational practices from siloed development to shared components will require a phased approach. Our rollout strategy addresses these concerns by beginning with select product lines and gradually expanding based on learnings from each implementation.


Our rollout strategy addresses these concerns through a phased approach, beginning with select product lines and gradually expanding based on learnings from each implementation.

### What are the key risks and how will you mitigate them?

We have identified several technical, organizational, and customer experience risks associated with our project. To mitigate performance overhead, we have implemented a zero-copy, memory-mapped execution environment and comprehensive performance testing. Network dependency is addressed by bundling fallback plugins for critical setup components. Plugin compatibility is managed through our versioned interface with shadow-versioning for backward compatibility. Security vulnerabilities are minimized through signed plugins, a capability-based security model, and sandbox execution.
On the organizational front, adoption resistance is mitigated by demonstrating clear ROI and involving key stakeholders early in the design process. Skill set gaps are addressed through comprehensive documentation and training programs. Governance challenges are managed through clear ownership boundaries and an established plugin review process.
For customer experience risks, update failures are prevented through gradual rollout with automated rollback capabilities. Inconsistent implementation is addressed through comprehensive guidelines and review processes. Setup delays are mitigated by optimizing plugin sizes and implementing parallel download strategies.

### Can this scale to millions of customers/devices?

Our architecture was designed from the ground up for massive scale. The DS2 team will create a dedicated infrastructure for plugin distribution, providing global edge caching with high availability. Our execution plan service will use a high-throughput system with auto-scaling capabilities designed to handle millions of requests per minute during peak periods. Binary plugins use differential updates to minimize bandwidth consumption, ensuring a minimal network footprint. The runtime engine requires less than 5MB of resident memory regardless of setup complexity, optimizing resource usage. Intelligent rate limiting and backoff strategies prevent thundering herd problems during mass device activations. For each SDK release, we will validate the system through rigorous simulation testing with models representing ten times our current device activation peaks, ensuring the architecture continues to scale elastically as demand grows.

For each SDK release, we will validate the system through rigorous simulation testing with models representing 10x our current device activation peaks, ensuring the architecture continues to scale elastically as demand grows.

### How will your SDK evolve over time?

In the near term, our focus for the next year includes expanding the plugin library to cover all common setup tasks. We aim to create advanced targeting capabilities based on user context and build developer tools to simplify plugin creation and testing. Additionally, we plan to implement automated quality assurance for contributed plugins.
Looking ahead to the mid-term, over the next two to three years, we intend to extend our capabilities to companion apps for mobile and web setup experiences. We will create machine learning systems to dynamically optimize flow sequences and build personalized setup experiences based on customer history. Furthermore, we aim to establish an open plugin marketplace for third-party developers.
Our long-term vision is to evolve from device setup to whole-home ecosystem management. We aspire to create predictive setup that anticipates customer needs and enable zero-UI setup through ambient intelligence.