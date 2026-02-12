mod common;

use helados_sofis_core::shared::auth::{create_jwt, verify_jwt, Role};
use uuid::Uuid;

// ═══════════════════════════════════════════════════════════
// Tests Unitarios — Auth / JWT / Role
// Patrón AAA: Arrange → Act → Assert
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod role_tests {
    use super::*;

    #[test]
    fn role_from_str_owner_valido() {
        // Arrange
        let input = "owner";

        // Act
        let role = Role::from_str(input).unwrap();

        // Assert
        assert_eq!(role, Role::Owner);
    }

    #[test]
    fn role_from_str_admin_valido() {
        // Arrange
        let input = "admin";

        // Act
        let role = Role::from_str(input).unwrap();

        // Assert
        assert_eq!(role, Role::Admin);
    }

    #[test]
    fn role_from_str_invalido_retorna_error() {
        // Arrange
        let input = "superadmin";

        // Act
        let result = Role::from_str(input);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn role_as_str_devuelve_texto_correcto() {
        // Arrange & Act & Assert
        assert_eq!(Role::Owner.as_str(), "owner");
        assert_eq!(Role::Admin.as_str(), "admin");
    }

    #[test]
    fn role_display_formato_correcto() {
        // Arrange & Act
        let owner_str = format!("{}", Role::Owner);
        let admin_str = format!("{}", Role::Admin);

        // Assert
        assert_eq!(owner_str, "owner");
        assert_eq!(admin_str, "admin");
    }

    #[test]
    fn role_ordering_owner_mayor_que_admin() {
        // Arrange
        let admin = Role::Admin;
        let owner = Role::Owner;

        // Act & Assert
        assert!(owner > admin);
        assert!(owner >= admin);
        assert!(admin < owner);
    }
}

#[cfg(test)]
mod jwt_tests {
    use super::*;

    const TEST_SECRET: &str = "test-secret-key-super-segura";

    #[test]
    fn crear_jwt_genera_token_valido() {
        // Arrange
        let user_id = Uuid::new_v4();
        let email = "test@helados.com";
        let role = Role::Owner;

        // Act
        let token = create_jwt(user_id, email, role, TEST_SECRET);

        // Assert
        assert!(token.is_ok());
        assert!(!token.unwrap().is_empty());
    }

    #[test]
    fn verificar_jwt_valido_devuelve_claims() {
        // Arrange
        let user_id = Uuid::new_v4();
        let email = "test@helados.com";
        let role = Role::Owner;
        let token = create_jwt(user_id, email, role, TEST_SECRET).unwrap();

        // Act
        let claims = verify_jwt(&token, TEST_SECRET);

        // Assert
        assert!(claims.is_ok());
        let claims = claims.unwrap();
        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.email, email);
        assert_eq!(claims.role, role);
    }

    #[test]
    fn verificar_jwt_con_secret_incorrecto_falla() {
        // Arrange
        let token = create_jwt(Uuid::new_v4(), "test@helados.com", Role::Admin, TEST_SECRET)
            .unwrap();

        // Act
        let result = verify_jwt(&token, "wrong-secret");

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn verificar_jwt_con_token_corrupto_falla() {
        // Arrange
        let token = "eyJhbGciOiJIUzI1NiJ9.corrupto.invalido";

        // Act
        let result = verify_jwt(token, TEST_SECRET);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn jwt_claims_preservan_rol_admin() {
        // Arrange
        let user_id = Uuid::new_v4();
        let email = "admin@helados.com";

        // Act
        let token = create_jwt(user_id, email, Role::Admin, TEST_SECRET).unwrap();
        let claims = verify_jwt(&token, TEST_SECRET).unwrap();

        // Assert
        assert_eq!(claims.role, Role::Admin);
    }

    #[test]
    fn jwt_claims_contienen_timestamps() {
        // Arrange
        let token =
            create_jwt(Uuid::new_v4(), "test@helados.com", Role::Owner, TEST_SECRET).unwrap();

        // Act
        let claims = verify_jwt(&token, TEST_SECRET).unwrap();

        // Assert
        assert!(claims.iat > 0);
        assert!(claims.exp > claims.iat);
        // El token expira en ~1 año
        let diff = claims.exp - claims.iat;
        assert_eq!(diff, 60 * 60 * 24 * 365);
    }
}

#[cfg(test)]
mod error_tests {
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use helados_sofis_core::shared::errors::AppError;

    #[tokio::test]
    async fn not_found_devuelve_404() {
        // Arrange
        let error = AppError::NotFound("recurso".into());

        // Act
        let response = error.into_response();

        // Assert
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn bad_request_devuelve_400() {
        // Arrange
        let error = AppError::BadRequest("dato inválido".into());

        // Act
        let response = error.into_response();

        // Assert
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn unauthorized_devuelve_401() {
        // Arrange
        let error = AppError::Unauthorized("sin token".into());

        // Act
        let response = error.into_response();

        // Assert
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn forbidden_devuelve_403() {
        // Arrange
        let error = AppError::Forbidden("sin permisos".into());

        // Act
        let response = error.into_response();

        // Assert
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn conflict_devuelve_409() {
        // Arrange
        let error = AppError::Conflict("duplicado".into());

        // Act
        let response = error.into_response();

        // Assert
        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn insufficient_stock_devuelve_409() {
        // Arrange
        let error = AppError::InsufficientStock(uuid::Uuid::new_v4());

        // Act
        let response = error.into_response();

        // Assert
        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn internal_devuelve_500() {
        // Arrange
        let error = AppError::Internal("algo falló".into());

        // Act
        let response = error.into_response();

        // Assert
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}

#[cfg(test)]
mod auth_user_tests {
    use helados_sofis_core::shared::auth::{AuthUser, Claims, Role};
    use uuid::Uuid;

    fn make_auth_user(role: Role) -> AuthUser {
        AuthUser(Claims {
            sub: Uuid::new_v4(),
            email: "test@helados.com".into(),
            role,
            iat: 0,
            exp: usize::MAX,
        })
    }

    #[test]
    fn require_role_admin_cuando_es_owner_ok() {
        // Arrange
        let user = make_auth_user(Role::Owner);

        // Act
        let result = user.require_role(Role::Admin);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn require_role_owner_cuando_es_admin_falla() {
        // Arrange
        let user = make_auth_user(Role::Admin);

        // Act
        let result = user.require_role(Role::Owner);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn require_owner_cuando_es_owner_ok() {
        // Arrange
        let user = make_auth_user(Role::Owner);

        // Act & Assert
        assert!(user.require_owner().is_ok());
    }

    #[test]
    fn require_owner_cuando_es_admin_falla() {
        // Arrange
        let user = make_auth_user(Role::Admin);

        // Act & Assert
        assert!(user.require_owner().is_err());
    }

    #[test]
    fn user_id_devuelve_uuid_correcto() {
        // Arrange
        let id = Uuid::new_v4();
        let user = AuthUser(Claims {
            sub: id,
            email: "test@helados.com".into(),
            role: Role::Admin,
            iat: 0,
            exp: usize::MAX,
        });

        // Act & Assert
        assert_eq!(user.user_id(), id);
    }

    #[test]
    fn role_devuelve_rol_correcto() {
        // Arrange
        let user = make_auth_user(Role::Owner);

        // Act & Assert
        assert_eq!(user.role(), Role::Owner);
    }
}
