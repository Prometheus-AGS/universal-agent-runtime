import { Component, type ReactNode } from "react";
import { SourceCodeBlock } from "./source-code-block";

interface LazyMarkdownBlockBoundaryProps {
  children: ReactNode;
  language: string;
  resetKey: string;
  source: string;
}

interface LazyMarkdownBlockBoundaryState {
  failed: boolean;
}

/** Keep an asynchronous block failure local to its source fence. */
export class LazyMarkdownBlockBoundary extends Component<
  LazyMarkdownBlockBoundaryProps,
  LazyMarkdownBlockBoundaryState
> {
  state: LazyMarkdownBlockBoundaryState = { failed: false };

  static getDerivedStateFromError(): LazyMarkdownBlockBoundaryState {
    return { failed: true };
  }

  componentDidCatch(): void {
    // The source fallback is intentional; renderer errors never replace sibling markdown.
  }

  componentDidUpdate(previous: LazyMarkdownBlockBoundaryProps): void {
    if (this.state.failed && previous.resetKey !== this.props.resetKey) {
      this.setState({ failed: false });
    }
  }

  render(): ReactNode {
    if (this.state.failed) {
      return (
        <SourceCodeBlock
          source={this.props.source}
          language={this.props.language}
          status="Preview unavailable; showing source"
        />
      );
    }

    return this.props.children;
  }
}
