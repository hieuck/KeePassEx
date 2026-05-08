/**
 * Global Error Boundary — catches React render errors and shows a friendly UI
 * instead of a blank black screen.
 */
import { Component, type ErrorInfo, type ReactNode } from 'react';

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error('[KeePassEx] Render error:', error, info.componentStack);
  }

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) return this.props.fallback;

      return (
        <div
          style={{
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            justifyContent: 'center',
            height: '100%',
            gap: 16,
            padding: 32,
            color: 'var(--color-text, #f9fafb)',
            background: 'var(--color-bg, #111827)',
          }}
        >
          <span style={{ fontSize: 48 }}>⚠️</span>
          <h2 style={{ fontSize: 18, fontWeight: 600, margin: 0 }}>Đã xảy ra lỗi</h2>
          <p
            style={{
              fontSize: 13,
              color: 'var(--color-text-secondary, #9ca3af)',
              textAlign: 'center',
              maxWidth: 400,
            }}
          >
            {this.state.error?.message || 'Unknown error'}
          </p>
          <button
            style={{
              padding: '8px 20px',
              background: 'var(--color-primary, #2563eb)',
              color: 'white',
              border: 'none',
              borderRadius: 8,
              cursor: 'pointer',
              fontSize: 14,
            }}
            onClick={() => this.setState({ hasError: false, error: null })}
          >
            Thử lại
          </button>
        </div>
      );
    }

    return this.props.children;
  }
}
