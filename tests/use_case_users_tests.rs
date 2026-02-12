mod common;

use std::sync::Arc;
use uuid::Uuid;

use common::mocks::*;
use helados_sofis_core::modules::users::application::{create_user, get_user, list_users, update_user};
use helados_sofis_core::modules::users::domain::entities::*;
use helados_sofis_core::shared::auth::Role;

// ═══════════════════════════════════════════════════════════
// Tests de Casos de Uso — Usuarios (con Mocks)
// Patrón AAA: Arrange → Act → Assert
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod crear_usuario_tests {
    use super::*;

    #[tokio::test]
    async fn crear_usuario_exitoso_cuando_email_no_existe() {
        // Arrange
        let mut mock = MockUserRepo::new();
        let owner_id = Uuid::new_v4();
        let expected_user = fake_user(Role::Admin);
        let expected_clone = expected_user.clone();

        mock.expect_find_by_email()
            .withf(|email| email == "nuevo@helados.com")
            .times(1)
            .returning(|_| Ok(None));

        mock.expect_create()
            .times(1)
            .returning(move |_, _| Ok(expected_clone.clone()));

        let repo: Arc<dyn helados_sofis_core::modules::users::domain::repositories::UserRepository> =
            Arc::new(mock);

        let dto = CreateUserDto {
            email: "nuevo@helados.com".into(),
            display_name: "Nuevo Usuario".into(),
            photo_url: None,
            role: Role::Admin,
            notes: None,
        };

        // Act
        let result = create_user::execute(&repo, dto, owner_id).await;

        // Assert
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.id, expected_user.id);
    }

    #[tokio::test]
    async fn crear_usuario_falla_cuando_email_ya_existe() {
        // Arrange
        let mut mock = MockUserRepo::new();
        let existing = fake_user(Role::Admin);

        mock.expect_find_by_email()
            .times(1)
            .returning(move |_| Ok(Some(existing.clone())));

        // create NO debería llamarse
        mock.expect_create().times(0);

        let repo: Arc<dyn helados_sofis_core::modules::users::domain::repositories::UserRepository> =
            Arc::new(mock);

        let dto = CreateUserDto {
            email: "existente@helados.com".into(),
            display_name: "Duplicado".into(),
            photo_url: None,
            role: Role::Admin,
            notes: None,
        };

        // Act
        let result = create_user::execute(&repo, dto, Uuid::new_v4()).await;

        // Assert
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("Ya existe"));
    }
}

#[cfg(test)]
mod obtener_usuario_tests {
    use super::*;

    #[tokio::test]
    async fn obtener_por_id_exitoso() {
        // Arrange
        let mut mock = MockUserRepo::new();
        let user = fake_user(Role::Owner);
        let user_id = user.id;
        let user_clone = user.clone();

        mock.expect_find_by_id()
            .withf(move |id| *id == user_id)
            .times(1)
            .returning(move |_| Ok(Some(user_clone.clone())));

        let repo: Arc<dyn helados_sofis_core::modules::users::domain::repositories::UserRepository> =
            Arc::new(mock);

        // Act
        let result = get_user::by_id(&repo, user_id).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, user_id);
    }

    #[tokio::test]
    async fn obtener_por_id_no_encontrado() {
        // Arrange
        let mut mock = MockUserRepo::new();
        let missing_id = Uuid::new_v4();

        mock.expect_find_by_id()
            .times(1)
            .returning(|_| Ok(None));

        let repo: Arc<dyn helados_sofis_core::modules::users::domain::repositories::UserRepository> =
            Arc::new(mock);

        // Act
        let result = get_user::by_id(&repo, missing_id).await;

        // Assert
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("no encontrado"));
    }

    #[tokio::test]
    async fn obtener_por_email_exitoso() {
        // Arrange
        let mut mock = MockUserRepo::new();
        let user = fake_user(Role::Admin);
        let user_clone = user.clone();

        mock.expect_find_by_email()
            .withf(|email| email == "admin@helados.com")
            .times(1)
            .returning(move |_| Ok(Some(user_clone.clone())));

        let repo: Arc<dyn helados_sofis_core::modules::users::domain::repositories::UserRepository> =
            Arc::new(mock);

        // Act
        let result = get_user::by_email(&repo, "admin@helados.com").await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, user.id);
    }

    #[tokio::test]
    async fn obtener_por_email_no_encontrado() {
        // Arrange
        let mut mock = MockUserRepo::new();

        mock.expect_find_by_email()
            .times(1)
            .returning(|_| Ok(None));

        let repo: Arc<dyn helados_sofis_core::modules::users::domain::repositories::UserRepository> =
            Arc::new(mock);

        // Act
        let result = get_user::by_email(&repo, "noexiste@helados.com").await;

        // Assert
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod listar_usuarios_tests {
    use super::*;

    #[tokio::test]
    async fn listar_todos_devuelve_usuarios() {
        // Arrange
        let mut mock = MockUserRepo::new();
        let users = vec![fake_user(Role::Owner), fake_user(Role::Admin)];
        let users_clone = users.clone();

        mock.expect_find_all()
            .times(1)
            .returning(move || Ok(users_clone.clone()));

        let repo: Arc<dyn helados_sofis_core::modules::users::domain::repositories::UserRepository> =
            Arc::new(mock);

        // Act
        let result = list_users::all(&repo).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn listar_activos_devuelve_solo_activos() {
        // Arrange
        let mut mock = MockUserRepo::new();
        let active = fake_user(Role::Admin);
        let active_clone = active.clone();

        mock.expect_find_active()
            .times(1)
            .returning(move || Ok(vec![active_clone.clone()]));

        let repo: Arc<dyn helados_sofis_core::modules::users::domain::repositories::UserRepository> =
            Arc::new(mock);

        // Act
        let result = list_users::active_only(&repo).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }
}

#[cfg(test)]
mod actualizar_usuario_tests {
    use super::*;

    #[tokio::test]
    async fn actualizar_usuario_existente_ok() {
        // Arrange
        let mut mock = MockUserRepo::new();
        let existing = fake_user(Role::Admin);
        let user_id = existing.id;
        let existing_clone = existing.clone();
        let mut updated = existing.clone();
        updated.display_name = "Nombre Actualizado".into();
        let updated_clone = updated.clone();

        mock.expect_find_by_id()
            .withf(move |id| *id == user_id)
            .times(1)
            .returning(move |_| Ok(Some(existing_clone.clone())));

        mock.expect_update()
            .times(1)
            .returning(move |_, _| Ok(updated_clone.clone()));

        let repo: Arc<dyn helados_sofis_core::modules::users::domain::repositories::UserRepository> =
            Arc::new(mock);

        let dto = UpdateUserDto {
            display_name: Some("Nombre Actualizado".into()),
            photo_url: None,
            role: None,
            active: None,
            notes: None,
        };

        // Act
        let result = update_user::execute(&repo, user_id, dto).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap().display_name, "Nombre Actualizado");
    }

    #[tokio::test]
    async fn actualizar_usuario_no_existente_falla() {
        // Arrange
        let mut mock = MockUserRepo::new();
        let missing_id = Uuid::new_v4();

        mock.expect_find_by_id()
            .times(1)
            .returning(|_| Ok(None));

        mock.expect_update().times(0);

        let repo: Arc<dyn helados_sofis_core::modules::users::domain::repositories::UserRepository> =
            Arc::new(mock);

        let dto = UpdateUserDto {
            display_name: Some("No Debería".into()),
            photo_url: None,
            role: None,
            active: None,
            notes: None,
        };

        // Act
        let result = update_user::execute(&repo, missing_id, dto).await;

        // Assert
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("no encontrado"));
    }
}
