use std::collections::BTreeSet;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    Safe,
    Sensitive,
    Dangerous,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionPolicy {
    pub action: String,
    pub risk: RiskLevel,
    pub required_permissions: BTreeSet<String>,
    pub confirmation_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecurityContext {
    pub granted_permissions: BTreeSet<String>,
    pub confirmed: bool,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SecurityError {
    #[error("action cannot be empty")]
    EmptyAction,
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    #[error("user confirmation required")]
    ConfirmationRequired,
}

pub fn authorize(policy: &ActionPolicy, context: &SecurityContext) -> Result<(), SecurityError> {
    if policy.action.trim().is_empty() {
        return Err(SecurityError::EmptyAction);
    }

    for permission in &policy.required_permissions {
        if !context.granted_permissions.contains(permission) {
            return Err(SecurityError::PermissionDenied(permission.clone()));
        }
    }

    if policy.confirmation_required && !context.confirmed {
        return Err(SecurityError::ConfirmationRequired);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> ActionPolicy {
        ActionPolicy {
            action: "camera.open".into(),
            risk: RiskLevel::Sensitive,
            required_permissions: BTreeSet::from(["camera".into()]),
            confirmation_required: false,
        }
    }

    fn context() -> SecurityContext {
        SecurityContext {
            granted_permissions: BTreeSet::from(["camera".into()]),
            confirmed: false,
        }
    }

    #[test]
    fn authorizes_granted_permission() {
        assert!(authorize(&policy(), &context()).is_ok());
    }

    #[test]
    fn rejects_missing_permission() {
        let mut context = context();
        context.granted_permissions.clear();

        assert_eq!(
            authorize(&policy(), &context),
            Err(SecurityError::PermissionDenied("camera".into()))
        );
    }

    #[test]
    fn requires_confirmation_when_policy_demands_it() {
        let mut policy = policy();
        policy.confirmation_required = true;

        assert_eq!(
            authorize(&policy, &context()),
            Err(SecurityError::ConfirmationRequired)
        );
    }

    #[test]
    fn confirmation_allows_confirmed_action() {
        let mut policy = policy();
        policy.confirmation_required = true;
        let mut context = context();
        context.confirmed = true;

        assert!(authorize(&policy, &context).is_ok());
    }

    #[test]
    fn rejects_empty_action() {
        let mut policy = policy();
        policy.action.clear();

        assert_eq!(authorize(&policy, &context()), Err(SecurityError::EmptyAction));
    }
}
