import { Component, type ErrorInfo, type ReactNode } from "react";

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

/**
 * React Error Boundary — catches rendering errors and shows a recovery UI.
 * Prevents the entire app from crashing to a white screen.
 */
export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    // Log to console for development debugging
    console.error("Ghost ErrorBoundary caught:", error, errorInfo);
  }

  handleReset = () => {
    this.setState({ hasError: false, error: null });
  };

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback;
      }

      return (
        <div className="flex flex-col items-center justify-center h-dvh bg-ghost-bg text-ghost-text p-8 gap-4">
          <div className="w-14 h-14 rounded-2xl bg-ghost-danger/10 flex items-center justify-center">
            <span className="text-2xl">⚠️</span>
          </div>
          <div className="text-center space-y-2 max-w-md">
            <h2 className="text-lg font-semibold">Algo salió mal</h2>
            <p className="text-sm text-ghost-text-dim/60">
              Ghost encontró un error inesperado. Puedes intentar reiniciar la vista.
            </p>
            {this.state.error && (
              <pre className="mt-3 p-3 bg-ghost-surface rounded-xl text-xs text-ghost-text-dim/50 text-left overflow-auto max-h-32">
                {this.state.error.message}
              </pre>
            )}
          </div>
          <button
            onClick={this.handleReset}
            className="px-4 py-2 rounded-xl bg-ghost-accent text-white text-sm font-medium hover:bg-ghost-accent-dim transition-colors"
          >
            Reiniciar vista
          </button>
        </div>
      );
    }

    return this.props.children;
  }
}
