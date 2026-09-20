---
outline: deep
---

# Participation Lifecycle

Welcome to the Mental Card Games project! Whether you are here for a practical project or a thesis, this page guides you through the full process—from selecting a topic, setting up your environment, checking in during development, up to your final code submission and grading.

We structure your journey into four clear phases to minimize friction and ensure a successful project.

## The Four-Phase Student Funnel

```mermaid
flowchart LR
    A["1. DISCOVERY<br>Find a topic"] --> B["2. ONBOARDING<br>Setup dev env"]
    B --> C["3. ACTIVE CODE<br>Regular syncs"]
    C --> D["4. WRAP-UP<br>Submit & Grade"]
```

## 1. Discovery & Topic Selection

The first step is identifying a technical backlog task or research area that excites you and aligns with your background.

1. **Explore the Backlog:** Review the open research and software development items listed in our [Student Milestones Backlog](/project/milestones).
2. **Connect with Supervisors & Project Members:** Reach out to the supervisors listed on the [Active Registry (People)](/organisational/people) page and other project members to get in touch. This is important to help supervisors better guide you to good ideas, refine project boundaries, and discuss the expected workload.
3. **Kick-off Presentation:** Once you decide on a milestone to work on, you are expected to hold a kick-off presentation. This presentation specifies the tasks you will work on, how you plan to solve the problems, or which research questions will be investigated in case of a thesis.

## 2. Onboarding & Environment Setup

Once your topic is finalized, you will establish your developer workspace. This phase aims to get you fully set up and familiar with the system architecture.

1. **Read the Blueprints:** Make sure you study:
   - The [System Design](/project/system-design) page to understand components, actor model data flows, system boundaries ...
   - The [Repository Components & Layout](/project/repository-components-layout) page to understand crate dependencies and the codebase structure.
2. **Set Up Your Toolchain:** Follow our step-by-step developer environment manual in the [Developer Setup Guide](/organisational/contribute).
3. **Join Communication Channels:** Ask your supervisor to invite you to our internal resources (e.g., Matrix or Next Cloud) and verify you have access to the GitHub organization.

## 3. Active Research & Coding

This is the main phase of your project. We promote an open, collaborative environment where students support each other.

* **Continuous Exchange with Supervisors:** Ongoing, active communication with your supervisors after the kick-off is critical to keep them up to date with your progress and ensure your project stays on track. It is also the best way to get help, receive guidance, and get unstuck if you face obstacles.
* **Regular Syncs & Peer Exchange:** While there is no rigid clock-in requirement, it is highly recommended to participate in regular exchange with other active students. A key opportunity for this is our regular **Mental Card Game Meeting**, which are advertised on our internal chat server (Matrix) and listed on the chair's website. These syncs are a great way to resolve shared engineering challenges and enjoy a collaborative peer environment.
* **Coordinate Dependencies:** Look at the [Active Registry (People)](/organisational/people) to see what other students are building. Students are expected to add themselves to this registry, though doing so is entirely voluntary to respect data privacy. Since the project uses a shared monorepo, early communication on API changes or protocol enhancements prevents merge conflicts.
* **Submit Iterative Pull Requests:** Avoid massive "end-of-semester" code dumps. Frequent, smaller pull requests (PRs) make review easier and help maintain stable compilation across the monorepo.

## 4. Project Wrap-up & Grading

As your project nears completion, you will transition to the final evaluation phase.

1. **Code Handover:** All code must be submitted via a final pull request on GitHub. Your code must format correctly (`cargo fmt`), compile with zero warnings (`cargo clippy`), and pass the existing workspace tests (`cargo test`).
2. **Documentation & Reports:** Ensure any new crates, modules, or features you build are documented in the code via rustdocs. Thesis students will submit their written report according to university regulations.
3. **Evaluation Metrics:** Your final score/grade is determined by a holistic evaluation of the following areas:
   - **Technical Correctness & Safety:** Code design, robustness, test coverage, and adherence to system invariants.
   - **Documentation Quality:** Complete code comments, clear commit logs, and helpful documentation updates.
   - **Supervision & Proactiveness:** Active participation in regular syncs, independence in solving problems, and communication.
   - **Research Depth:** The scientific innovation and analytical depth demonstrated in the project (especially for thesis work).
4. **Knowledge Preservation:** Once graded, your research report and code contribution will be archived to ensure future generations of students can build upon your accomplishments!
