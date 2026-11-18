import { usePeerMessageField, useArtifactField, useMemberArtifactIds, usePeerMessageIds } from "@/platform/entities";
import { formatDateTime } from "../../model/format";
import { Collect } from "./collect";
import { Disclosure } from "./disclosure";
import { OutputView } from "./output-view";
import { addIfInvolved, NO_IDS, useMessageInvolves, type RunContext } from "./run-hooks";
import { humanizeToken } from "./run-model";
import { MemberRoleName, TaskTitle } from "./task-parts";

interface MemberScope extends RunContext {
  memberId: string;
}

function PeerMessageRow({ workspace, teamId, messageId, memberId }: MemberScope & { messageId: string }) {
  const sender = usePeerMessageField(workspace, teamId, messageId, "senderMemberId");
  const recipient = usePeerMessageField(workspace, teamId, messageId, "recipientMemberId");
  const mode = usePeerMessageField(workspace, teamId, messageId, "mode");
  const payload = usePeerMessageField(workspace, teamId, messageId, "payload");
  const acceptedAt = usePeerMessageField(workspace, teamId, messageId, "acceptedAt");
  const delivery = usePeerMessageField(workspace, teamId, messageId, "delivery");
  const outgoing = sender === memberId;
  return (
    <li className="min-w-0 space-y-1 rounded-md bg-background/70 p-3 text-sm">
      <p className="break-words">
        <span className="font-medium">{outgoing ? "Sent to " : "Received from "}</span>
        {outgoing && recipient ? <MemberRoleName workspace={workspace} teamId={teamId} memberId={recipient} /> : null}
        {!outgoing && sender ? <MemberRoleName workspace={workspace} teamId={teamId} memberId={sender} /> : null}
        {mode ? <span className="text-xs text-muted-foreground"> ({humanizeToken(mode)})</span> : null}
      </p>
      {payload?.text ? <p className="break-words whitespace-pre-wrap">{payload.text}</p> : <p className="text-muted-foreground">No text.</p>}
      <p className="text-xs text-muted-foreground">
        {formatDateTime(acceptedAt)}
        {delivery ? `, delivery ${humanizeToken(delivery.status)}` : ""}
      </p>
    </li>
  );
}

/** Peer messages the member sent or received, collapsed by default. */
export function MemberMessages({ workspace, teamId, memberId }: MemberScope) {
  const messageIds = usePeerMessageIds(workspace, teamId);
  return (
    <Collect
      ids={messageIds}
      context={{ workspace, teamId, memberId }}
      useValue={useMessageInvolves}
      reduce={addIfInvolved}
      initial={NO_IDS}
    >
      {(involved) => (
        <Disclosure label="Peer messages" count={involved.length}>
          {involved.length === 0 ? (
            <p className="text-sm text-muted-foreground">No peer message was sent to or by this member.</p>
          ) : (
            <ul className="space-y-2">
              {involved.map((messageId) => (
                <PeerMessageRow key={messageId} workspace={workspace} teamId={teamId} messageId={messageId} memberId={memberId} />
              ))}
            </ul>
          )}
        </Disclosure>
      )}
    </Collect>
  );
}

function ArtifactRow({ workspace, teamId, artifactId }: RunContext & { artifactId: string }) {
  const taskId = useArtifactField(workspace, teamId, artifactId, "taskId");
  const content = useArtifactField(workspace, teamId, artifactId, "content");
  const createdAt = useArtifactField(workspace, teamId, artifactId, "createdAt");
  return (
    <li className="min-w-0 space-y-1 rounded-md bg-background/70 p-3">
      <p className="text-xs break-words text-muted-foreground">
        {taskId ? (
          <>
            Task: <TaskTitle workspace={workspace} teamId={teamId} taskId={taskId} />,{" "}
          </>
        ) : null}
        {formatDateTime(createdAt)}
      </p>
      <OutputView label="Content" value={content} />
    </li>
  );
}

/** Artifacts the member produced, collapsed by default. */
export function MemberArtifacts({ workspace, teamId, memberId }: MemberScope) {
  const artifactIds = useMemberArtifactIds(workspace, teamId, memberId);
  return (
    <Disclosure label="Artifacts" count={artifactIds.length}>
      {artifactIds.length === 0 ? (
        <p className="text-sm text-muted-foreground">This member has produced no artifacts.</p>
      ) : (
        <ul className="space-y-2">
          {artifactIds.map((artifactId) => (
            <ArtifactRow key={artifactId} workspace={workspace} teamId={teamId} artifactId={artifactId} />
          ))}
        </ul>
      )}
    </Disclosure>
  );
}
