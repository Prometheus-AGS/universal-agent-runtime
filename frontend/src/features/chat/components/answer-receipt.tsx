import { memo } from "react";

import { formatReceiptTime, modelOriginLabel, type ModelReceipt } from "@/features/chat/model/model-receipt";
import { useAnswerReceipt } from "@/features/chat/model/use-answer-receipt";

/**
 * One quiet line under an assistant answer: the model that actually answered (as the provider
 * reported it), why the runtime chose it, and, for the system default, since when it became the
 * default. It is the runtime's statement, never the requested model. Nothing renders until the
 * runtime sends a receipt, so a refused or failed turn shows none.
 */
export function AnswerReceipt({ messageId }: { messageId: string }) {
  const receipt = useAnswerReceipt(messageId);
  return receipt ? <AnswerReceiptLine receipt={receipt} /> : null;
}

const AnswerReceiptLine = memo(function AnswerReceiptLine({ receipt }: { receipt: ModelReceipt }) {
  const time = receipt.since ? formatReceiptTime(receipt.since) : null;
  return (
    <p
      data-testid="answer-receipt"
      className="mt-1 mb-0 flex flex-wrap items-baseline gap-x-1.5 gap-y-0.5 font-mono text-[11px] leading-snug text-muted-foreground"
    >
      <span>Answered by</span>
      <span data-testid="answer-receipt-model" className="text-foreground">
        {`${receipt.provider}/${receipt.model}`}
      </span>
      {receipt.origin && (
        <>
          <span aria-hidden="true">·</span>
          <span data-testid="answer-receipt-origin">{modelOriginLabel(receipt.origin)}</span>
        </>
      )}
      {receipt.since && time && (
        <>
          <span aria-hidden="true">·</span>
          <span data-testid="answer-receipt-since">
            <span>since </span>
            <time dateTime={receipt.since} title={new Date(receipt.since).toLocaleString()}>
              {time}
            </time>
          </span>
        </>
      )}
    </p>
  );
});
