import { Component, type ErrorInfo, type ReactNode } from "react";

type ErrorBoundaryProps = {
  children: ReactNode;
};

type ErrorBoundaryState = {
  error: Error | null;
  stack?: string;
};

export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  state: ErrorBoundaryState = {
    error: null,
  };

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { error };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    this.setState({
      error,
      stack: info.componentStack ?? undefined,
    });
  }

  render() {
    if (!this.state.error) {
      return this.props.children;
    }

    return (
      <main className="app-shell">
        <section className="usage-panel">
          <header className="panel-header">
            <div>
              <h1>Money Like Water</h1>
              <p>Interface rendering failed</p>
            </div>
          </header>
          <p className="notice error">{this.state.error.message}</p>
          {this.state.stack && <pre className="error-stack">{this.state.stack}</pre>}
        </section>
      </main>
    );
  }
}
