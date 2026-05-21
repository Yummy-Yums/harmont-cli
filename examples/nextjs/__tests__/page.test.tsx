import { render, screen } from "@testing-library/react";
import Page from "../app/page";

test("renders headline", () => {
  render(<Page />);
  expect(screen.getByText(/Hello from Harmont/)).toBeInTheDocument();
});
