import React from "react";
import renderer from "react-test-renderer";
import { ProposalState } from "@nebgov/sdk";
import { ProposalStateBadge } from "../ProposalStateBadge";

const STATES = [
  ProposalState.Pending,
  ProposalState.Active,
  ProposalState.Succeeded,
  ProposalState.Defeated,
  ProposalState.Queued,
  ProposalState.Executed,
  ProposalState.Cancelled,
  ProposalState.Expired,
];

describe("ProposalStateBadge", () => {
  it.each(STATES)("matches snapshot for state %s", (state) => {
    const tree = renderer.create(<ProposalStateBadge state={state} />).toJSON();
    expect(tree).toMatchSnapshot();
  });

  it.each(STATES)("renders a non-empty badge for state %s", (state) => {
    const tree = renderer.create(<ProposalStateBadge state={state} />).toJSON();
    expect(tree).toBeTruthy();
    expect(tree).toHaveProperty("type", "span");
    expect(tree).toHaveProperty("children");
    expect(Array.isArray(tree.children)).toBe(true);
    expect(tree.children.length).toBeGreaterThan(0);
  });
});
