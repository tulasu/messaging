defmodule Messaging.Application do
  # See https://hexdocs.pm/elixir/Application.html
  # for more information on OTP Applications
  @moduledoc false

  use Application

  @impl true
  def start(_type, _args) do
    children = [
      Messaging.Repo,
      {DNSCluster, query: Application.get_env(:messaging, :dns_cluster_query) || :ignore},
      {Phoenix.PubSub, name: Messaging.PubSub}
      # Start a worker by calling: Messaging.Worker.start_link(arg)
      # {Messaging.Worker, arg}
    ]

    Supervisor.start_link(children, strategy: :one_for_one, name: Messaging.Supervisor)
  end
end
