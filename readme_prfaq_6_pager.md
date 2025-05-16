# PRFAQ: Plugin-Based OOBE Architecture

## Press Release

### Amazon Launches OOBE SDK to Transform Device Setup Experiences

SEATTLE – Amazon today announced a groundbreaking new architecture for out-of-box experiences (OOBE) across its device portfolio. The new plugin-based architecture fundamentally reimagines how setup experiences are built, deployed, and maintained, addressing long-standing challenges in the consumer electronics industry.

"This represents a paradigm shift in how we deliver first impressions to our customers," said [EXECUTIVE NAME], VP of Device Experience at Amazon. "By transforming our OOBE from static firmware to dynamic plugins, we're creating setup experiences that are consistent, updatable, and continuously improving—even for devices already in customer homes."

The new architecture dramatically improves development velocity, enabling teams to deliver experience improvements in days rather than months. Early trials show setup completion rates improved by 15%, while support contacts related to device setup have decreased by over 20%.

"Customer expectations for smart, adaptive experiences continue to rise," added [EXECUTIVE NAME]. "Our plugin-based architecture closes the gap between these expectations and our ability to meet them, while simultaneously reducing operational costs and creating new opportunities for post-setup engagement."

The architecture will begin rolling out to select devices in Q3, with portfolio-wide implementation expected by mid-next year.

## Frequently Asked Questions

### What problem does this new architecture solve?

Our new architecture addresses several fundamental challenges with traditional out-of-box experiences:

**Each device has its own OOBE software:** Currently, we maintain hundreds of different OOBE paths across our device portfolio. Each product line independently develops its setup flow, resulting in duplicated effort and inconsistent experiences. Our plugin architecture replaces this fragmented approach with a unified runtime engine and shared component library, dramatically reducing redundant development while ensuring consistent behavior. [Support this statement with data, for example: total number of distinct OOBE codepaths across product lines, developer hours spent reimplementing similar functionality, percentage of code duplication between product lines]

**Multiple operating systems require support:** Supporting multiple platforms (FOS, Vega, AOSP) currently forces teams to maintain parallel implementations of the same functionality. Our new architecture implements a platform-agnostic binary interface with cross-language ABI compatibility, allowing identical business logic to run across all platforms without recompilation or platform-specific code paths. [Support this statement with data, for example: number of platforms supported, lines of code dedicated to platform-specific logic, engineering hours spent maintaining separate implementations]

**Product variations create complexity:** Even within product lines, we must account for headless devices, multi-modal interactions, and entry-level vs. premium experiences. The current approach bundles all possible paths into monolithic packages, creating bloated deployments. Our architecture introduces dynamic module loading that delivers precisely the compiled native modules each device configuration needs, reducing package sizes by up to 80%. [Support this statement with data, for example: current deployment package sizes, percentage of unused code in typical deployments, loading time differences between monolithic vs. modular approaches]

**Duplicated common tasks waste resources:** Each team currently rebuilds fundamental tasks like device login, account linking, device registration, file copying, system feature enablement, and service restarts. Our plugin architecture centralizes these operations into canonical implementations maintained by domain experts, eliminating duplication while improving quality and consistency. [Support this statement with data, for example: number of teams implementing their own versions of these common components, defect rates in common components vs. specialized functionality, engineering hours spent maintaining redundant implementations]

**OOBE remains frozen after shipping:** Traditional OOBE workflows are hardcoded into firmware or bundled in system images, making them impossible to update without full firmware releases. Our architecture decouples OOBE from firmware through dynamically loadable plugins, allowing continuous improvement even for devices already in customer homes or warehouses. [Support this statement with data, for example: average time between OOBE improvements, percentage of devices in homes running outdated OOBE versions, customer impact metrics for devices unable to receive OOBE updates]

**Upsell during setup creates friction:** Current monetization approaches force upsell attempts during initial setup when customers are focused on getting devices operational. This creates friction and increases abandonment rates. Our architecture introduces the SCOOBE framework (Second Chance OOBE) that can intelligently schedule promotional interactions after the device is fully operational, improving conversion while reducing setup friction. [Support this statement with data, for example: setup abandonment rates at upsell screens, time spent on upsell screens vs. functional setup screens, conversion rates during setup vs. post-setup engagement]

**Inconsistent experiences damage brand perception:** Users currently encounter different visual languages and interaction patterns across our product portfolio, creating confusion and undermining brand cohesion. Our architecture enforces experience consistency through a shared pattern library and platform-adaptive rendering framework, ensuring users develop transferable mental models across all Amazon devices. [Support this statement with data, for example: customer confusion metrics across multi-device households, support contacts related to inconsistent experiences, brand perception metrics across product lines]

**Fragmented metrics limit optimization:** Each product line currently implements its own instrumentation, making cross-portfolio analysis nearly impossible. Our architecture includes an integrated telemetry framework that automatically collects hundreds of standardized metrics across all plugins without requiring developer implementation, creating unprecedented analytical capabilities. [Support this statement with data, for example: number of incompatible telemetry systems in use, percentage of important user journeys that cannot be tracked across products, engineering hours spent implementing redundant analytics]

**A/B testing and feature variation complexity:** The current architecture makes experimentation and feature management prohibitively complex due to feature variations, OS differences, device capabilities (headless/multimodal), and geographic restrictions (such as codec availability in certain regions). Our plugin system enables a comprehensive testing framework that dynamically composes features for specific device configurations and regions at runtime, while allowing developers to easily switch execution profiles during development. This dramatically simplifies pre-production validation and reduces the risk of shipping incompatible or legally non-compliant features. [Support this statement with data, for example: number of test configurations currently required to validate a feature across the product portfolio, time spent ensuring regional compliance, incidents of non-compliant features reaching production]

### How does the SDK solve these problems?

The core issues we face stem from duplicated effort, lack of reusability, and fragmented ownership. Multiple teams across the company are solving the same problems in isolation, often without knowing that solutions already exist. Our OOBE SDK addresses these challenges through domain-specific components and architectural innovations:

**1. Unifying device-specific OOBE software** Our SDK replaces hundreds of product-specific implementations with a single unified runtime engine and modular plugin framework. This shared architecture allows all products to leverage common components while still enabling customization for specific needs. Rather than building setup flows from scratch, teams compose them from pre-built, well-tested modules, reducing development effort by up to 60% while ensuring consistent quality. This dramatically reduces the redundant development currently happening across product lines.

**2. Supporting multiple operating systems** Through our platform-agnostic binary interface with cross-language ABI compatibility, we enable identical business logic to run across all platforms without recompilation or platform-specific code paths. This eliminates the need for teams to maintain parallel implementations of the same functionality for different operating systems. The interface provides guaranteed compatibility across FOS, Vega, and AOSP through a well-defined type system and shadow-versioning techniques.

**3. Solving product variation complexity** Each product variation receives its tailored execution plan from the cloud at runtime, eliminating the need to bundle all possible paths into the firmware. This enables precise customization for headless devices, multi-modal experiences, entry-level or premium features without bloating package sizes with unused code. Teams define their configuration profiles and the system dynamically delivers only what each device needs.

**4. Eliminating duplicated common tasks** By centralizing core components under domain expert ownership, we eliminate the redundant implementation of critical functionality across product lines. For example, WiFi configuration code currently exists in over X different codebases, each with its own bugs and edge-case handling. Our SDK provides a single, thoroughly tested implementation maintained by networking specialists that all products can leverage, dramatically reducing development time while improving reliability. Similar efficiencies apply to:

- **Device identity and authentication** - Built and maintained by domain experts in the eCommerce Foundation Identity team who understand security protocols, token management, and account verification at a depth that product teams cannot match
- **Account linking and provisioning** - Developed once by the Device Management team who already maintains our device fleet infrastructure but currently has no control over the onboarding flows that populate it
- **Network setup and zero-touch onboarding** - Owned by the Frustration-Free Setup team with their sophisticated WiFi configuration logic that's currently reimplemented—often with critical bugs—by each product team
- **System-level operations** - Standardized under the Device Software Services team, preventing dangerous implementations for tasks like file operations and service management
- **Legal compliance screens** - Governed by the Legal team rather than being hardcoded by individual product groups, eliminating compliance risk from inconsistent implementations

**5. Enabling updatable OOBE through dynamic execution plans** Our dynamic execution plan system completely transforms how setup experiences evolve after shipping. OOBE launches with a default plan and immediately after network connectivity is established, checks for an updated execution plan from the cloud that's specific to the product, version, region, device type, and user segment. It then downloads any necessary plugins and executes the latest flow, ensuring even devices manufactured months ago receive the most current experience. This solves the "frozen OOBE" problem without requiring firmware updates.

**6. Creating a platform for post-setup engagement** Our SCOOBE (Second Chance OOBE) framework allows for intelligent scheduling of promotional interactions after the device is fully operational, addressing the upsell-during-setup problem. By providing a mechanism to engage users at appropriate moments after they're already enjoying their device, we can improve conversion rates while reducing setup abandonment. SCOOBE extends far beyond simple upsell opportunities - it enables numerous post-setup optimizations:

- **Delayed system maintenance** - Postpone resource-intensive operations like Over-the-Air updates until periods of detected inactivity
- **Contextual feature education** - Introduce advanced features only after detecting meaningful usage patterns
- **Intelligent data collection** - Schedule periodic customer feedback surveys or usage analytics after specific milestones (e.g., X days of ownership)
- **Opportunistic engagement** - Present relevant content or subscription offers during detected idle periods
- **Progressive device integration** - Stage the introduction of ecosystem connections to other Amazon devices based on usage patterns
- **Seasonal feature highlights** - Time the introduction of holiday-specific features or content based on calendar events

The SCOOBE framework turns device setup from a one-time event into an evolving relationship, allowing us to thoughtfully sequence interactions that might otherwise overwhelm users during initial configuration. However, successfully leveraging this capability requires cross-team alignment on timing and strategy for these extended interactions.

**7. Creating consistent user experiences** The SDK creates a framework for cohesive customer journeys without dictating exact visual designs. Rather than forcing rigid screen templates, we provide common interaction models, navigation patterns, and functional flows that can be styled according to each product's needs. This preserves brand distinctiveness while ensuring that fundamental user actions like connecting to WiFi, signing in, or accepting terms work consistently across the portfolio. The result is reduced customer confusion, improved cross-device ownership, and measurably faster setup completion as customers apply mental models from one device to another.

**8. Standardizing metrics collection** Our built-in instrumentation library automatically sends console logs, errors, information, crashes, and user interactions to a centralized cloud-based metrics system. We also provide no-code ready-to-use function attributes that plugin developers can apply to automatically collect predefined metrics without writing any instrumentation code. This creates unprecedented visibility into user journeys and abandonment patterns across the entire portfolio.

**9. Enabling seamless A/B testing** Our dynamic execution plan system fundamentally solves the A/B testing challenge. By designing the SDK to fetch and execute different plans based on device type, region, product version, geographic location and other factors, we enable seamless experimentation without firmware changes. Product teams can define multiple execution plans that vary specific components or flows, then remotely control which customers receive each variant through our cloud distribution system. This allows for rapid experimentation and data-driven optimization previously impossible with firmware-embedded experiences.

Without a common SDK, each product team ends up owning their own OOBE stack—building setup flows from scratch, duplicating infrastructure, and making inconsistent UX decisions. Teams like Prime Video are forced to show account linking or terms pages during setup simply because they do not have access to reusable components built elsewhere.

The case is clear: our SDK enables shared ownership of core capabilities, reduces redundant work, enforces consistency, and gives product teams a reliable foundation on which to build truly differentiated experiences. To succeed, we've designed it around modular, domain-owned components with well-defined contracts—and we're committed to making it the default path forward for all device and app setup flows.


