package auth

import (
	"context"
	"errors"
	"strings"

	domain "voc-go-backend/internal/domain/user"
	"voc-go-backend/internal/infrastructure/security"
)

// Service handles authentication use cases.
type Service struct {
	users       domain.Repository
	decryptor   *security.RSADecryptor
	pwdVerifier security.PasswordVerifier
	tokenSvc    *security.TokenService
}

// NewService builds a new auth Service.
func NewService(
	users domain.Repository,
	decryptor *security.RSADecryptor,
	pwdVerifier security.PasswordVerifier,
	tokenSvc *security.TokenService,
) *Service {
	return &Service{
		users:       users,
		decryptor:   decryptor,
		pwdVerifier: pwdVerifier,
		tokenSvc:    tokenSvc,
	}
}

// Login validates credentials and issues a token.
func (s *Service) Login(ctx context.Context, req LoginRequest) (*LoginResponse, error) {
	authType := strings.ToUpper(strings.TrimSpace(req.AuthType))
	if authType != "" && authType != "ACCOUNT" {
		return nil, errors.New("暂不支持该认证方式")
	}
	if strings.TrimSpace(req.ClientID) == "" {
		return nil, errors.New("客户端ID不能为空")
	}
	if strings.TrimSpace(req.Username) == "" {
		return nil, errors.New("用户名不能为空")
	}
	if strings.TrimSpace(req.Password) == "" {
		return nil, errors.New("密码不能为空")
	}

	rawPassword, err := s.decryptor.DecryptBase64(req.Password)
	if err != nil {
		return nil, errors.New("密码解密失败")
	}

	user, err := s.users.GetByUsername(ctx, req.Username)
	if err != nil {
		return nil, err
	}
	// 用户名或密码不正确（与 Java 提示保持一致）
	if user == nil {
		return nil, errors.New("用户名或密码不正确")
	}

	ok, err := s.pwdVerifier.Verify(rawPassword, user.Password)
	if err != nil {
		return nil, err
	}
	if !ok {
		return nil, errors.New("用户名或密码不正确")
	}

	if !user.IsEnabled() {
		return nil, errors.New("此账号已被禁用，如有疑问，请联系管理员")
	}

	token, err := s.tokenSvc.Generate(user.ID)
	if err != nil {
		return nil, err
	}
	return &LoginResponse{
		Token:    token,
		UserID:   user.ID,
		Username: user.Username,
		Nickname: user.Nickname,
	}, nil
}
