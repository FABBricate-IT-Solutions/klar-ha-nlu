import { Component, type ErrorInfo, type ReactNode } from "react";
import { Button } from "@/components/ui/button";

type Props = { children: ReactNode };
type State = { error: Error | null };

/** Catch render failures so a bad deep link does not blank the whole UI. */
export class ErrorBoundary extends Component<Props, State> {
  state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  componentDidCatch(error: Error, info: ErrorInfo): void {
    console.error("Klar UI render failed", error, info.componentStack);
  }

  render(): ReactNode {
    if (!this.state.error) return this.props.children;
    return (
      <div className="page">
        <div className="card danger" role="alert">
          <h1>Something broke on this page</h1>
          <p className="muted">{this.state.error.message || "Unknown error"}</p>
          <Button
            type="button"
            onClick={() => {
              this.setState({ error: null });
              window.location.hash = "#/";
              window.location.reload();
            }}
          >
            Reload
          </Button>
        </div>
      </div>
    );
  }
}
