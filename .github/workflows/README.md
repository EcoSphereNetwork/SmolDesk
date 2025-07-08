# Reusable GitHub Actions Workflows

This directory contains a collection of reusable GitHub Actions workflows that can be used as templates for your projects.

## Available Workflows

### 1. Project Automation (`project-automation.yml`)
Handles project management automation:
- Issue and PR triage
- Welcome messages for first-time contributors
- Project board automation
- Auto-assignment of issues/PRs

Usage:
```yaml
jobs:
  project-automation:
    uses: ./.github/workflows/project-automation.yml
    with:
      project-name: 'Main Project'
      triage-label: 'needs-review'
      welcome-message: 'Thanks for contributing!'
      auto-assign: true
    secrets:
      PROJECT_TOKEN: ${{ secrets.PROJECT_TOKEN }}
```

### 2. Security Scanning (`security.yml`)
Comprehensive security analysis:
- CodeQL analysis
- SAST (Static Application Security Testing)
- Dependency vulnerability scanning
- Container security scanning
- Custom security issue creation

Usage:
```yaml
jobs:
  security:
    uses: ./.github/workflows/security.yml
    with:
      languages: '["python", "javascript"]'
      severity-level: 'high'
      enable-dependency-scan: true
      enable-container-scan: true
    secrets:
      SNYK_TOKEN: ${{ secrets.SNYK_TOKEN }}
      SONAR_TOKEN: ${{ secrets.SONAR_TOKEN }}
```

### 3. Dependency Management (`dependency-management.yml`)
Automated dependency updates:
- Multiple package manager support (poetry, pip, conda)
- Scheduled updates
- Automated PR creation
- Optional auto-merge
- Detailed update reports

Usage:
```yaml
jobs:
  dependencies:
    uses: ./.github/workflows/dependency-management.yml
    with:
      package-manager: 'poetry'
      update-schedule: '0 0 * * 1'
      auto-merge: true
      labels: 'dependencies,automated'
    secrets:
      RENOVATE_TOKEN: ${{ secrets.RENOVATE_TOKEN }}
```

### 4. Code Quality (`code-quality.yml`)
Comprehensive code quality checks:
- Multi-version Python testing
- Style checking (ruff, black)
- Type checking (mypy)
- Complexity analysis
- Coverage reporting
- Detailed quality reports

Usage:
```yaml
jobs:
  quality:
    uses: ./.github/workflows/code-quality.yml
    with:
      python-versions: '["3.9", "3.10", "3.11"]'
      enable-type-check: true
      max-complexity: 10
      coverage-threshold: 85
    secrets:
      CODECOV_TOKEN: ${{ secrets.CODECOV_TOKEN }}
```

### 5. Docker Build (`docker-build.yml`)
Container image building and publishing:
- Multi-platform support
- Build caching
- Registry authentication
- Metadata handling
- Automated tagging

Usage:
```yaml
jobs:
  build:
    uses: ./.github/workflows/docker-build.yml
    with:
      image-name: 'myorg/myapp'
      platforms: 'linux/amd64,linux/arm64'
      push: true
    secrets:
      REGISTRY_USERNAME: ${{ secrets.DOCKER_USERNAME }}
      REGISTRY_PASSWORD: ${{ secrets.DOCKER_PASSWORD }}
```

### 6. Deployment (`deploy.yml`)
Flexible deployment automation:
- Multiple deployment methods
- Infrastructure as Code support
- Cloud provider integration
- Environment management
- Deployment verification

Usage:
```yaml
jobs:
  deploy:
    uses: ./.github/workflows/deploy.yml
    with:
      environment: 'production'
      use-terraform: true
      terraform-workspace: 'prod'
    secrets:
      AWS_ACCESS_KEY_ID: ${{ secrets.AWS_ACCESS_KEY_ID }}
      AWS_SECRET_ACCESS_KEY: ${{ secrets.AWS_SECRET_ACCESS_KEY }}
```

### 7. Create Issue on Workflow Failure (create-issue-on-failure.yml)
Handles the automatic creation of GitHub issues when a workflow fails. This workflow ensures that any failed workflow is documented as an issue in the repository for easy tracking and resolution.
Features:

- Detects failed workflows.
- Automatically creates an issue with details of the failure.
- Includes failure metadata like workflow name, run ID, and commit information.


Usage:
```yaml
jobs:
  create-issue:
    uses: ./.github/workflows/create-issue-on-failure.yml
    with:
      issue-title: '[Workflow Failure] ${{ github.workflow }}'
      issue-body: |
        ## Workflow Failure Details
        - **Workflow Name**: ${{ github.workflow }}
        - **Run ID**: ${{ github.run_id }}
        - **Commit**: ${{ github.sha }}
        - **Branch**: ${{ github.ref }}
        - **Triggered By**: @${{ github.actor }}
        
        [View Logs](https://github.com/${{ github.repository }}/actions/runs/${{ github.run_id }})
      labels: 'failure, automation'
    secrets:
      GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}

```

## Required Secrets

Configure these secrets based on the workflows you use:

1. `GITHUB_TOKEN`: Automatically provided by GitHub
2. `PROJECT_TOKEN`: For project board operations
3. `SNYK_TOKEN`: For [Snyk](https://app.snyk.io/login) security scanning
4. `SONAR_TOKEN`: For [SonarCloud](https://www.sonarsource.com/products/sonarcloud/signup-free/) analysis
5. `CODECOV_TOKEN`: For [Codecov](https://about.codecov.io/codecov-free-trial/) coverage reporting
6. `RENOVATE_TOKEN`: For [Renovate](https://github.com/marketplace/renovate) dependency updates 
7. `AWS_ACCESS_KEY_ID` & `AWS_SECRET_ACCESS_KEY`: For AWS deployments
8. `DOCKER_USERNAME` & `DOCKER_PASSWORD`: For container registry

## Best Practices

1. **Workflow Selection**
   - Choose only the workflows you need
   - Customize inputs based on your requirements
   - Keep workflows modular and focused

2. **Security**
   - Store sensitive data in secrets
   - Use environment protection rules
   - Enable required status checks
   - Implement branch protection

3. **Maintenance**
   - Regularly update workflow versions
   - Monitor workflow execution times
   - Review and adjust thresholds
   - Keep dependencies updated

4. **Testing**
   - Test workflows in a fork first
   - Use matrix testing when possible
   - Set appropriate timeouts
   - Enable detailed logging when needed

## Troubleshooting

1. **Permission Issues**
   - Check workflow permissions in repository settings
   - Verify secret availability
   - Review token scopes

2. **Fork Considerations**
   - Use `pull_request_target` for fork access to secrets
   - Enable fork workflow settings
   - Configure appropriate permissions

3. **Performance**
   - Use build caching
   - Implement matrix job strategies
   - Optimize job dependencies
   - Monitor execution times
